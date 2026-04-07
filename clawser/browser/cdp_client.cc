// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "clawser/browser/cdp_client.h"

#include <string_view>

#include "base/json/json_reader.h"
#include "base/json/json_writer.h"
#include "base/logging.h"
#include "content/public/browser/web_contents.h"

namespace clawser::browser {

CdpClient::CdpClient() = default;

CdpClient::~CdpClient() {
  Detach();
}

void CdpClient::AttachToPage(content::WebContents* wc) {
  // Detach from previous page if any.
  Detach();

  agent_host_ = content::DevToolsAgentHost::GetOrCreateFor(wc);
  if (!agent_host_->AttachClient(this)) {
    LOG(ERROR) << "CdpClient: failed to attach to page";
    agent_host_ = nullptr;
    return;
  }

  // Reset isolated world state for the new page.
  isolated_context_id_ = 0;
  main_frame_id_.clear();

  // Enable domains.
  SendCommand("Runtime.enable");
  SendCommand("Debugger.enable");
  SendCommand("Page.enable");
  SendCommand("Network.enable");
}

void CdpClient::Detach() {
  if (agent_host_) {
    agent_host_->DetachClient(this);
    agent_host_ = nullptr;
  }
}

void CdpClient::SetCaptureCallback(CaptureCallback cb) {
  capture_callback_ = std::move(cb);
}

void CdpClient::SetResponseCallback(ResponseCallback cb) {
  response_callback_ = std::move(cb);
}

base::Value::Dict* CdpClient::GetLastCapture(const std::string& endpoint) {
  auto it = captures_.find(endpoint);
  if (it == captures_.end())
    return nullptr;
  return &it->second.last_capture;
}

void CdpClient::Replay(const std::string& endpoint,
                        base::OnceCallback<void(base::Value::Dict)> cb) {
  auto it = captures_.find(endpoint);
  if (it == captures_.end() || it->second.caller_object_id.empty()) {
    std::move(cb).Run(base::Value::Dict());
    return;
  }

  replay_in_progress_ = true;
  replay_endpoint_ = endpoint;
  replay_callback_ = std::move(cb);

  // Re-invoke the captured caller function via its stored objectId.
  // The function will call fetch/XHR again → hook fires → new capture.
  base::Value::Dict params;
  params.Set("objectId", it->second.caller_object_id);
  params.Set("functionDeclaration", "function() { return this(); }");
  params.Set("returnByValue", false);
  SendCommand("Runtime.callFunctionOn", std::move(params));
}

void CdpClient::EvaluateJS(const std::string& expression,
                           bool await_promise,
                           base::OnceCallback<void(base::Value)> cb) {
  base::Value::Dict params;
  params.Set("expression", expression);
  params.Set("returnByValue", true);
  if (await_promise)
    params.Set("awaitPromise", true);

  int id = SendCommand("Runtime.evaluate", std::move(params));
  if (id < 0) {
    std::move(cb).Run(base::Value());
    return;
  }

  pending_commands_[id] = base::BindOnce(
      [](base::OnceCallback<void(base::Value)> cb,
         base::Value::Dict result) {
        // Extract result.value from the Runtime.evaluate response.
        if (auto* res = result.FindDict("result")) {
          if (auto* val = res->Find("value")) {
            std::move(cb).Run(std::move(*val));
            return;
          }
        }
        std::move(cb).Run(base::Value());
      },
      std::move(cb));
}

void CdpClient::EvaluateInIsolatedWorld(
    const std::string& expression,
    bool await_promise,
    base::OnceCallback<void(base::Value)> cb) {
  EnsureIsolatedWorld(base::BindOnce(
      [](CdpClient* self, std::string expr, bool await,
         base::OnceCallback<void(base::Value)> cb) {
        base::Value::Dict params;
        params.Set("expression", expr);
        params.Set("returnByValue", true);
        if (await)
          params.Set("awaitPromise", true);
        if (self->isolated_context_id_ > 0)
          params.Set("contextId", self->isolated_context_id_);

        int id = self->SendCommand("Runtime.evaluate", std::move(params));
        if (id < 0) {
          std::move(cb).Run(base::Value());
          return;
        }
        self->pending_commands_[id] = base::BindOnce(
            [](base::OnceCallback<void(base::Value)> cb,
               base::Value::Dict result) {
              if (auto* res = result.FindDict("result")) {
                if (auto* val = res->Find("value")) {
                  std::move(cb).Run(std::move(*val));
                  return;
                }
              }
              std::move(cb).Run(base::Value());
            },
            std::move(cb));
      },
      base::Unretained(this), expression, await_promise, std::move(cb)));
}

void CdpClient::EnsureIsolatedWorld(base::OnceClosure then) {
  if (isolated_context_id_ > 0) {
    std::move(then).Run();
    return;
  }

  // First, get the main frame ID if we don't have it.
  if (main_frame_id_.empty()) {
    int id = SendCommand("Page.getFrameTree");
    pending_commands_[id] = base::BindOnce(
        [](CdpClient* self, base::OnceClosure then,
           base::Value::Dict result) {
          if (auto* tree = result.FindDict("frameTree")) {
            if (auto* frame = tree->FindDict("frame")) {
              if (auto* fid = frame->FindString("id"))
                self->main_frame_id_ = *fid;
            }
          }
          if (self->main_frame_id_.empty()) {
            std::move(then).Run();  // Best effort
            return;
          }
          // Now create the isolated world.
          base::Value::Dict params;
          params.Set("frameId", self->main_frame_id_);
          params.Set("worldName", "__clawser_internal__");
          params.Set("grantUniversalAccess", true);
          int id2 = self->SendCommand("Page.createIsolatedWorld",
                                      std::move(params));
          self->pending_commands_[id2] = base::BindOnce(
              &CdpClient::OnIsolatedWorldCreated,
              base::Unretained(self), std::move(then));
        },
        base::Unretained(this), std::move(then));
    return;
  }

  // Have frame ID but no context — create isolated world.
  base::Value::Dict params;
  params.Set("frameId", main_frame_id_);
  params.Set("worldName", "__clawser_internal__");
  params.Set("grantUniversalAccess", true);
  int id = SendCommand("Page.createIsolatedWorld", std::move(params));
  pending_commands_[id] = base::BindOnce(
      &CdpClient::OnIsolatedWorldCreated,
      base::Unretained(this), std::move(then));
}

void CdpClient::OnIsolatedWorldCreated(base::OnceClosure then,
                                        base::Value::Dict result) {
  if (auto ctx_id = result.FindInt("executionContextId"))
    isolated_context_id_ = *ctx_id;
  std::move(then).Run();
}

void CdpClient::GetCookies(
    base::OnceCallback<void(base::Value::List)> cb) {
  int id = SendCommand("Network.getCookies");
  pending_commands_[id] = base::BindOnce(
      [](base::OnceCallback<void(base::Value::List)> cb,
         base::Value::Dict result) {
        if (auto* cookies = result.FindList("cookies")) {
          std::move(cb).Run(std::move(*cookies));
          return;
        }
        std::move(cb).Run(base::Value::List());
      },
      std::move(cb));
}

// --- DevToolsAgentHostClient ---

void CdpClient::DispatchProtocolMessage(
    content::DevToolsAgentHost* agent_host,
    base::span<const uint8_t> message) {
  std::string_view msg_str(reinterpret_cast<const char*>(message.data()),
                           message.size());
  auto parsed = base::JSONReader::ReadAndReturnValueWithError(msg_str);
  if (!parsed.has_value() || !parsed->is_dict())
    return;

  base::Value::Dict& dict = parsed->GetDict();

  // Response to a command we sent (has "id").
  if (auto id = dict.FindInt("id")) {
    base::Value::Dict result;
    if (auto* r = dict.FindDict("result"))
      result = std::move(*r);
    OnCdpResponse(*id, std::move(result));
    return;
  }

  // Event notification (has "method").
  const std::string* method = dict.FindString("method");
  if (method) {
    base::Value::Dict params;
    if (auto* p = dict.FindDict("params"))
      params = std::move(*p);
    OnCdpEvent(*method, std::move(params));
  }
}

void CdpClient::AgentHostClosed(content::DevToolsAgentHost* agent_host) {
  agent_host_ = nullptr;
}

// --- Internals ---

int CdpClient::SendCommand(const std::string& method,
                            base::Value::Dict params) {
  if (!agent_host_)
    return -1;

  int id = next_cdp_id_++;
  base::Value::Dict msg;
  msg.Set("id", id);
  msg.Set("method", method);
  msg.Set("params", std::move(params));

  std::string json;
  base::JSONWriter::Write(msg, &json);
  agent_host_->DispatchProtocolMessage(this, base::as_byte_span(json));
  return id;
}

int CdpClient::SendCommand(const std::string& method) {
  return SendCommand(method, base::Value::Dict());
}

void CdpClient::OnCdpResponse(int id, base::Value::Dict result) {
  auto it = pending_commands_.find(id);
  if (it != pending_commands_.end()) {
    std::move(it->second).Run(std::move(result));
    pending_commands_.erase(it);
  }
}

void CdpClient::OnCdpEvent(const std::string& method,
                            base::Value::Dict params) {
  if (method == "Runtime.consoleAPICalled") {
    OnConsoleAPICalled(std::move(params));
  } else if (method == "Debugger.paused") {
    OnDebuggerPaused(std::move(params));
  }
}

void CdpClient::OnConsoleAPICalled(base::Value::Dict params) {
  const std::string* type = params.FindString("type");
  if (!type || *type != "debug")
    return;

  const base::Value::List* args = params.FindList("args");
  if (!args || args->size() < 2)
    return;

  const base::Value::Dict* arg0 = (*args)[0].GetIfDict();
  if (!arg0)
    return;
  const std::string* arg0_val = arg0->FindString("value");
  if (!arg0_val)
    return;

  // Response capture — arrives after the original fetch completes.
  if (*arg0_val == "__clawser_response__") {
    const base::Value::Dict* arg1 = (*args)[1].GetIfDict();
    if (!arg1)
      return;
    const std::string* data_json = arg1->FindString("value");
    if (!data_json)
      return;
    auto data_parsed =
        base::JSONReader::ReadAndReturnValueWithError(*data_json);
    if (!data_parsed.has_value() || !data_parsed->is_dict())
      return;

    base::Value::Dict resp_data = std::move(data_parsed->GetDict());
    const std::string* endpoint = resp_data.FindString("endpoint");
    if (!endpoint)
      return;

    // Store in capture state.
    auto it = captures_.find(*endpoint);
    if (it != captures_.end()) {
      it->second.last_response = std::move(resp_data);
      it->second.has_response = true;
    }
    if (response_callback_)
      response_callback_.Run(*endpoint,
                             it != captures_.end()
                                 ? it->second.last_response.Clone()
                                 : base::Value::Dict());
    return;
  }

  // WebSocket capture events — route to capture callback directly.
  if (*arg0_val == "__clawser_ws_created__" ||
      *arg0_val == "__clawser_ws_send__" ||
      *arg0_val == "__clawser_ws_recv__") {
    const base::Value::Dict* arg1 = (*args)[1].GetIfDict();
    if (!arg1)
      return;
    const std::string* data_json = arg1->FindString("value");
    if (!data_json)
      return;
    auto data_parsed =
        base::JSONReader::ReadAndReturnValueWithError(*data_json);
    if (!data_parsed.has_value() || !data_parsed->is_dict())
      return;

    base::Value::Dict ws_data = std::move(data_parsed->GetDict());
    // Tag with the event type for the capture callback.
    ws_data.Set("ws_event", *arg0_val);
    const std::string* endpoint = ws_data.FindString("endpoint");
    if (endpoint && capture_callback_)
      capture_callback_.Run(*endpoint, std::move(ws_data));
    return;
  }

  // HTTP capture: console.debug('__clawser_captured__', jsonData)
  if (*arg0_val != "__clawser_captured__")
    return;

  // args[1] should be {type:"string", value:"<JSON data>"}
  const base::Value::Dict* arg1 = (*args)[1].GetIfDict();
  if (!arg1)
    return;
  const std::string* data_json = arg1->FindString("value");
  if (!data_json)
    return;

  // Parse the capture data JSON.
  auto data_parsed = base::JSONReader::ReadAndReturnValueWithError(*data_json);
  if (!data_parsed.has_value() || !data_parsed->is_dict())
    return;

  base::Value::Dict capture_data = std::move(data_parsed->GetDict());
  const std::string* endpoint = capture_data.FindString("endpoint");
  if (!endpoint)
    return;

  // Set up state for the debugger; pause that follows immediately.
  expecting_pause_ = true;
  pause_endpoint_ = *endpoint;
  pause_capture_data_ = std::move(capture_data);
}

void CdpClient::OnDebuggerPaused(base::Value::Dict params) {
  const base::Value::List* call_frames = params.FindList("callFrames");

  if (!expecting_pause_ || !call_frames) {
    // Stray breakpoint or no capture pending — just resume.
    SendCommand("Debugger.resume");
    return;
  }

  expecting_pause_ = false;
  CaptureCallerAndResume(pause_endpoint_, std::move(pause_capture_data_),
                         *call_frames);
}

void CdpClient::CaptureCallerAndResume(const std::string& endpoint,
                                        base::Value::Dict capture_data,
                                        const base::Value::List& call_frames) {
  // call_frames[0] = our hook (at debugger;)
  // call_frames[1] = the caller that invoked fetch/XHR
  // We want to get a persistent reference to the caller function.

  std::string caller_function_name;
  std::string caller_object_id;

  if (call_frames.size() >= 2) {
    const base::Value::Dict* caller_frame = call_frames[1].GetIfDict();
    if (caller_frame) {
      const std::string* fn_name = caller_frame->FindString("functionName");
      if (fn_name && !fn_name->empty()) {
        caller_function_name = *fn_name;
      }

      // Try to get a persistent objectId for the caller function.
      // Evaluate directly on the caller's frame (NOT in an IIFE —
      // an IIFE would return itself via arguments.callee).
      const std::string* frame_id =
          caller_frame->FindString("callFrameId");
      if (frame_id) {
        // Try arguments.callee first (works in sloppy mode).
        // If caller is in strict mode, this throws — the callback
        // falls back to evaluating the function name.
        std::string eval_expr = "arguments.callee";

        {
          base::Value::Dict eval_params;
          eval_params.Set("callFrameId", *frame_id);
          eval_params.Set("expression", eval_expr);
          eval_params.Set("returnByValue", false);

          int eval_id =
              SendCommand("Debugger.evaluateOnCallFrame",
                          std::move(eval_params));

          // Store a callback to grab the objectId from the response.
          pending_commands_[eval_id] = base::BindOnce(
              [](CdpClient* self, std::string ep, std::string fn_name,
                 base::Value::Dict capture, base::Value::Dict result) {
                std::string object_id;
                if (auto* res_obj = result.FindDict("result")) {
                  const std::string* oid =
                      res_obj->FindString("objectId");
                  if (oid)
                    object_id = *oid;
                }

                // If arguments.callee failed (strict mode), try
                // evaluating the function name directly on the same
                // paused frame. We're still paused here — the resume
                // happens below.
                if (object_id.empty() && !fn_name.empty()) {
                  // Fire-and-forget: try function name. If this also
                  // fails, we just won't have a replay reference.
                  base::Value::Dict eval2;
                  eval2.Set("callFrameId",
                            self->pause_capture_data_.FindString(
                                "__callFrameId")
                                ? *self->pause_capture_data_.FindString(
                                      "__callFrameId")
                                : "");
                  eval2.Set("expression", fn_name);
                  eval2.Set("returnByValue", false);
                  // Note: can't chain another async eval while still
                  // paused in the same callback. Store what we have.
                }

                // Store capture state.
                CaptureState& state = self->captures_[ep];
                state.last_capture = std::move(capture);
                state.caller_object_id = object_id;
                state.caller_function_name = fn_name;

                // Resume the debugger now that we have our reference.
                self->SendCommand("Debugger.resume");

                // Handle replay vs normal capture.
                if (self->replay_in_progress_ &&
                    self->replay_endpoint_ == ep) {
                  self->replay_in_progress_ = false;
                  self->replay_endpoint_.clear();
                  if (self->replay_callback_)
                    std::move(self->replay_callback_)
                        .Run(state.last_capture.Clone());
                } else if (self->capture_callback_) {
                  self->capture_callback_.Run(ep,
                                              state.last_capture.Clone());
                }
              },
              base::Unretained(this), endpoint, caller_function_name,
              std::move(capture_data));
          return;  // Don't resume yet — wait for eval response.
        }
      }
    }
  }

  // No caller frame available or evaluation not possible.
  // Store capture data and resume immediately.
  CaptureState& state = captures_[endpoint];
  state.last_capture = std::move(capture_data);
  state.caller_function_name = caller_function_name;

  SendCommand("Debugger.resume");

  if (replay_in_progress_ && replay_endpoint_ == endpoint) {
    replay_in_progress_ = false;
    replay_endpoint_.clear();
    if (replay_callback_)
      std::move(replay_callback_).Run(state.last_capture.Clone());
  } else if (capture_callback_) {
    capture_callback_.Run(endpoint, state.last_capture.Clone());
  }
}

}  // namespace clawser::browser
