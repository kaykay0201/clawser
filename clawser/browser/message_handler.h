// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_MESSAGE_HANDLER_H_
#define CLAWSER_BROWSER_MESSAGE_HANDLER_H_

#include <memory>
#include <string>

#include "base/memory/raw_ptr.h"
#include "base/memory/weak_ptr.h"
#include "base/values.h"
#include "net/base/io_buffer.h"
#include "net/socket/stream_socket.h"
#include "net/traffic_annotation/network_traffic_annotation.h"

namespace clawser::browser {

class BrowserController;

// Reads JSON commands from a TCP socket, dispatches to BrowserController,
// writes JSON responses back. All command dispatch happens on the browser
// UI thread.
class MessageHandler {
 public:
  explicit MessageHandler(BrowserController* controller);
  ~MessageHandler();

  // Start the TCP command loop. Takes ownership of the connected socket.
  void StartTcpLoop(std::unique_ptr<net::StreamSocket> socket);

 private:
  // Kick off an async read on the TCP socket.
  void ReadMore();
  void OnReadComplete(int result);

  // Process complete lines from the read buffer.
  void ProcessLines();

  // Dispatch a single JSON command (runs on UI thread).
  void HandleMessage(std::string json_line);

  // Write a JSON response to the TCP socket.
  void TcpWrite(const std::string& data);
  void OnWriteComplete(int result);

  // Reply helpers.
  void Reply(int id, base::Value::Dict result);
  void ReplyError(int id, const std::string& error);

  // Command handlers.
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

  // TCP socket for JSON protocol.
  std::unique_ptr<net::StreamSocket> socket_;
  scoped_refptr<net::IOBufferWithSize> read_buf_;
  std::string line_buffer_;  // Accumulates partial reads until newline.

  base::WeakPtrFactory<MessageHandler> weak_factory_{this};
};

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_MESSAGE_HANDLER_H_
