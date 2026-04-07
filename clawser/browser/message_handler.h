// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_MESSAGE_HANDLER_H_
#define CLAWSER_BROWSER_MESSAGE_HANDLER_H_

#include <memory>
#include <string>

#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "base/synchronization/lock.h"
#include "base/threading/thread.h"
#include "base/values.h"

namespace clawser::browser {

class BrowserController;

// Reads JSON commands from stdin, dispatches to BrowserController,
// writes JSON responses to stdout. All command dispatch happens on
// the browser UI thread.
class MessageHandler {
 public:
  explicit MessageHandler(BrowserController* controller);
  ~MessageHandler();

  // Starts the stdin read loop on a dedicated thread.
  // Must be called from the UI thread after browser is initialized.
  void StartStdinLoop();

 private:
  // Runs on the stdin reader thread — blocking reads.
  void StdinReadLoop();

  // Runs on the UI thread — dispatches a single JSON command.
  void HandleMessage(std::string json_line);

  // Writes a JSON response to stdout. Thread-safe.
  void Reply(int id, base::Value::Dict result);
  void ReplyError(int id, const std::string& error);

  // Command handlers
  void HandleNavigate(int id, const base::Value::Dict& params);
  void HandleWatch(int id, const base::Value::Dict& params);
  void HandleWait(int id, const base::Value::Dict& params);
  void HandleReplay(int id, const base::Value::Dict& params);
  void HandleEval(int id, const base::Value::Dict& params);
  void HandleLastCapture(int id, const base::Value::Dict& params);
  void HandleFetch(int id, const base::Value::Dict& params);
  void HandleCookies(int id, const base::Value::Dict& params);
  void HandleWebSocket(int id, const base::Value::Dict& params);
  void HandleWsSend(int id, const base::Value::Dict& params);
  void HandleWsRecv(int id, const base::Value::Dict& params);
  void HandleWsClose(int id, const base::Value::Dict& params);
  void HandleShutdown(int id);

  raw_ptr<BrowserController> controller_;
  base::Thread stdin_thread_;
  base::Lock stdout_lock_;

  base::WeakPtrFactory<MessageHandler> weak_factory_{this};
};

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_MESSAGE_HANDLER_H_
