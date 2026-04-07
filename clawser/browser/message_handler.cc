// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "clawser/browser/message_handler.h"

#include <iostream>

#include "base/functional/bind.h"
#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/logging.h"
#include "base/strings/stringprintf.h"
#include "base/synchronization/lock.h"
#include "base/task/single_thread_task_runner.h"
#include "clawser/browser/browser_controller.h"
#include "content/public/browser/browser_thread.h"

namespace clawser::browser {

// Defined in clawser_browser_main.cc — uses raw WriteFile to bypass CRT
// stdout issues on Windows when spawned with piped handles.
void WriteJsonLine(const std::string& json_with_newline);

MessageHandler::MessageHandler(BrowserController* controller)
    : controller_(controller), stdin_thread_("ClawserStdinReader") {}

MessageHandler::~MessageHandler() {
  if (stdin_thread_.IsRunning())
    stdin_thread_.Stop();
}

void MessageHandler::StartStdinLoop() {
  stdin_thread_.Start();
  stdin_thread_.task_runner()->PostTask(
      FROM_HERE,
      base::BindOnce(&MessageHandler::StdinReadLoop, base::Unretained(this)));
}

void MessageHandler::StdinReadLoop() {
  // NOTE: This runs on the ClawserStdinReader thread, NOT the UI thread.
  // We must NOT call weak_factory_.GetWeakPtr() here — WeakPtrFactory is
  // bound to the UI thread. Using base::Unretained is safe because the
  // destructor calls stdin_thread_.Stop() before destroying |this|.
  std::string line;
  while (std::getline(std::cin, line)) {
    if (line.empty())
      continue;
    content::GetUIThreadTaskRunner({})->PostTask(
        FROM_HERE, base::BindOnce(&MessageHandler::HandleMessage,
                                  base::Unretained(this), std::move(line)));
  }
  // stdin closed — parent process died or sent shutdown
  content::GetUIThreadTaskRunner({})->PostTask(
      FROM_HERE,
      base::BindOnce(&MessageHandler::HandleShutdown,
                     base::Unretained(this), /*id=*/0));
}

void MessageHandler::HandleMessage(std::string json_line) {
  DCHECK_CURRENTLY_ON(content::BrowserThread::UI);

  auto parsed = base::JSONReader::ReadAndReturnValueWithError(json_line);
  if (!parsed.has_value() || !parsed->is_dict()) {
    LOG(ERROR) << "Invalid JSON: " << json_line;
    return;
  }

  const base::Value::Dict& cmd = parsed->GetDict();
  int id = cmd.FindInt("id").value_or(0);
  const std::string* cmd_type = cmd.FindString("cmd");
  if (!cmd_type) {
    ReplyError(id, "missing 'cmd' field");
    return;
  }

  if (*cmd_type == "navigate") {
    HandleNavigate(id, cmd);
  } else if (*cmd_type == "watch") {
    HandleWatch(id, cmd);
  } else if (*cmd_type == "wait") {
    HandleWait(id, cmd);
  } else if (*cmd_type == "replay") {
    HandleReplay(id, cmd);
  } else if (*cmd_type == "js") {
    HandleEval(id, cmd);
  } else if (*cmd_type == "fetch") {
    HandleFetch(id, cmd);
  } else if (*cmd_type == "last_capture") {
    HandleLastCapture(id, cmd);
  } else if (*cmd_type == "cookies") {
    HandleCookies(id, cmd);
  } else if (*cmd_type == "websocket") {
    HandleWebSocket(id, cmd);
  } else if (*cmd_type == "ws_send") {
    HandleWsSend(id, cmd);
  } else if (*cmd_type == "ws_recv") {
    HandleWsRecv(id, cmd);
  } else if (*cmd_type == "ws_close") {
    HandleWsClose(id, cmd);
  } else if (*cmd_type == "shutdown") {
    HandleShutdown(id);
  } else {
    ReplyError(id, "unknown command: " + *cmd_type);
  }
}

void MessageHandler::Reply(int id, base::Value::Dict result) {
  result.Set("id", id);
  result.Set("ok", true);

  std::string json;
  base::JSONWriter::Write(result, &json);
  json += "\n";

  base::AutoLock lock(stdout_lock_);
  WriteJsonLine(json);
}

void MessageHandler::ReplyError(int id, const std::string& error) {
  base::Value::Dict result;
  result.Set("id", id);
  result.Set("ok", false);
  result.Set("error", error);

  std::string json;
  base::JSONWriter::Write(result, &json);
  json += "\n";

  base::AutoLock lock(stdout_lock_);
  WriteJsonLine(json);
}

void MessageHandler::HandleNavigate(int id, const base::Value::Dict& params) {
  const std::string* url = params.FindString("url");
  if (!url) {
    ReplyError(id, "missing 'url' field");
    return;
  }

  std::string page_id = controller_->Navigate(*url);
  if (page_id.empty()) {
    ReplyError(id, "navigation failed");
    return;
  }

  base::Value::Dict result;
  result.Set("page_id", page_id);
  Reply(id, std::move(result));
}

void MessageHandler::HandleWatch(int id, const base::Value::Dict& params) {
  const std::string* endpoint = params.FindString("endpoint");
  if (!endpoint) {
    ReplyError(id, "missing 'endpoint' field");
    return;
  }

  std::string watch_id = controller_->AddWatch(*endpoint);
  base::Value::Dict result;
  result.Set("watch_id", watch_id);
  Reply(id, std::move(result));
}

void MessageHandler::HandleWait(int id, const base::Value::Dict& params) {
  const std::string* watch_id = params.FindString("watch_id");
  if (!watch_id) {
    ReplyError(id, "missing 'watch_id' field");
    return;
  }

  uint32_t timeout_ms = static_cast<uint32_t>(
      params.FindInt("timeout_ms").value_or(30000));

  controller_->WaitForCapture(
      *watch_id, timeout_ms,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id,
             bool timed_out, base::Value::Dict data) {
            if (!handler)
              return;
            if (timed_out) {
              handler->ReplyError(id, "wait timeout");
              return;
            }
            base::Value::Dict result;
            result.Set("captured", std::move(data));
            handler->Reply(id, std::move(result));
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleReplay(int id, const base::Value::Dict& params) {
  const std::string* watch_id = params.FindString("watch_id");
  if (!watch_id) {
    ReplyError(id, "missing 'watch_id' field");
    return;
  }

  controller_->ReplayCapture(
      *watch_id,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id,
             base::Value::Dict data) {
            if (!handler)
              return;
            if (data.empty()) {
              handler->ReplyError(id, "replay failed: no caller reference");
              return;
            }
            base::Value::Dict result;
            result.Set("captured", std::move(data));
            handler->Reply(id, std::move(result));
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleEval(int id, const base::Value::Dict& params) {
  const std::string* page_id = params.FindString("page_id");
  const std::string* code = params.FindString("code");

  if (!page_id || !code) {
    ReplyError(id, "missing 'page_id' or 'code' field");
    return;
  }

  controller_->ExecuteJS(
      *page_id, *code,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id,
             base::Value result) {
            if (!handler)
              return;
            base::Value::Dict response;
            response.Set("result", std::move(result));
            handler->Reply(id, std::move(response));
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleLastCapture(int id,
                                       const base::Value::Dict& params) {
  const std::string* watch_id = params.FindString("watch_id");
  if (!watch_id) {
    ReplyError(id, "missing 'watch_id' field");
    return;
  }

  auto* data = controller_->GetLastCapture(*watch_id);
  base::Value::Dict result;
  if (data)
    result.Set("captured", data->Clone());
  Reply(id, std::move(result));
}

void MessageHandler::HandleFetch(int id, const base::Value::Dict& params) {
  const std::string* method = params.FindString("method");
  const std::string* url = params.FindString("url");
  if (!url) {
    ReplyError(id, "missing 'url' field");
    return;
  }

  std::string m = method ? *method : "GET";
  const base::Value::Dict* headers = params.FindDict("headers");
  const std::string* body = params.FindString("body");
  uint32_t timeout_ms = static_cast<uint32_t>(
      params.FindInt("timeout_ms").value_or(30000));

  controller_->FetchRequest(
      m, *url, headers, body, timeout_ms,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id,
             int status, base::Value::Dict resp_headers,
             std::string body, std::string final_url) {
            if (!handler)
              return;
            base::Value::Dict result;
            result.Set("status", status);
            result.Set("headers", std::move(resp_headers));
            result.Set("body", std::move(body));
            result.Set("url", std::move(final_url));
            handler->Reply(id, std::move(result));
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleCookies(int id, const base::Value::Dict& params) {
  const std::string* url = params.FindString("url");
  std::string url_str = url ? *url : "";

  controller_->GetCookies(
      url_str,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id,
             base::Value::List cookies) {
            if (!handler)
              return;
            base::Value::Dict result;
            result.Set("cookies", std::move(cookies));
            handler->Reply(id, std::move(result));
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleWebSocket(int id,
                                     const base::Value::Dict& params) {
  const std::string* url = params.FindString("url");
  const std::string* page_id = params.FindString("page_id");
  if (!url) {
    ReplyError(id, "missing 'url' field");
    return;
  }
  // Default to page p0 if not specified.
  std::string pid = page_id ? *page_id : "p0";

  controller_->OpenWebSocket(
      pid, *url,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id,
             std::string ws_id) {
            if (!handler)
              return;
            if (ws_id.empty()) {
              handler->ReplyError(id, "websocket open failed");
              return;
            }
            base::Value::Dict result;
            result.Set("ws_id", ws_id);
            handler->Reply(id, std::move(result));
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleWsSend(int id, const base::Value::Dict& params) {
  const std::string* ws_id = params.FindString("ws_id");
  const std::string* data = params.FindString("data");
  if (!ws_id || !data) {
    ReplyError(id, "missing 'ws_id' or 'data' field");
    return;
  }

  controller_->SendWebSocket(
      *ws_id, *data,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id, bool ok) {
            if (!handler)
              return;
            if (!ok) {
              handler->ReplyError(id, "ws_send failed");
              return;
            }
            handler->Reply(id, base::Value::Dict());
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleWsRecv(int id, const base::Value::Dict& params) {
  const std::string* ws_id = params.FindString("ws_id");
  if (!ws_id) {
    ReplyError(id, "missing 'ws_id' field");
    return;
  }

  uint32_t timeout_ms = static_cast<uint32_t>(
      params.FindInt("timeout_ms").value_or(30000));

  controller_->RecvWebSocket(
      *ws_id, timeout_ms,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id,
             std::string data) {
            if (!handler)
              return;
            base::Value::Dict result;
            result.Set("data", data);
            handler->Reply(id, std::move(result));
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleWsClose(int id, const base::Value::Dict& params) {
  const std::string* ws_id = params.FindString("ws_id");
  if (!ws_id) {
    ReplyError(id, "missing 'ws_id' field");
    return;
  }

  controller_->CloseWebSocket(
      *ws_id,
      base::BindOnce(
          [](base::WeakPtr<MessageHandler> handler, int id, bool ok) {
            if (!handler)
              return;
            handler->Reply(id, base::Value::Dict());
          },
          weak_factory_.GetWeakPtr(), id));
}

void MessageHandler::HandleShutdown(int id) {
  base::Value::Dict result;
  Reply(id, std::move(result));
  controller_->Shutdown();
}

}  // namespace clawser::browser
