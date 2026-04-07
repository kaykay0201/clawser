// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CLAWSER_BROWSER_NET_WEBSOCKET_H_
#define CLAWSER_BROWSER_NET_WEBSOCKET_H_

#include <string>
#include <vector>

#include "base/functional/callback.h"
#include "base/timer/timer.h"
#include "mojo/public/cpp/bindings/receiver.h"
#include "mojo/public/cpp/bindings/remote.h"
#include "mojo/public/cpp/system/data_pipe.h"
#include "mojo/public/cpp/system/simple_watcher.h"
#include "services/network/public/mojom/network_context.mojom.h"
#include "services/network/public/mojom/websocket.mojom.h"

namespace clawser::browser {

// C++ WebSocket client using Chromium's network service (Mojo).
// Uses the same TLS/H2 stack as the browser — fully antidetect.
// All methods must be called on the UI thread.
class NetWebSocket : public ::network::mojom::WebSocketHandshakeClient,
                     public ::network::mojom::WebSocketClient {
 public:
  NetWebSocket();
  ~NetWebSocket() override;

  // Connect to a WebSocket URL via the network service.
  void Connect(const GURL& url,
               ::network::mojom::NetworkContext* network_context,
               const url::Origin& origin,
               base::OnceCallback<void(bool ok)> cb);

  // Send a text message.
  void Send(const std::string& data, base::OnceCallback<void(bool ok)> cb);

  // Receive a text message (with timeout). Empty string = timeout/closed.
  void Recv(uint32_t timeout_ms,
            base::OnceCallback<void(std::string data)> cb);

  // Close the connection.
  void Close();

  bool is_connected() const { return connected_; }

 private:
  struct RecvWaiter {
    RecvWaiter();
    ~RecvWaiter();
    base::OnceCallback<void(std::string)> callback;
    base::OneShotTimer timeout;
  };

  // WebSocketHandshakeClient
  void OnOpeningHandshakeStarted(
      ::network::mojom::WebSocketHandshakeRequestPtr request) override;
  void OnFailure(const std::string& message,
                 int32_t net_error,
                 int32_t response_code) override;
  void OnConnectionEstablished(
      mojo::PendingRemote<::network::mojom::WebSocket> socket,
      mojo::PendingReceiver<::network::mojom::WebSocketClient>
          client_receiver,
      ::network::mojom::WebSocketHandshakeResponsePtr response,
      mojo::ScopedDataPipeConsumerHandle readable,
      mojo::ScopedDataPipeProducerHandle writable) override;

  // WebSocketClient
  void OnDataFrame(bool fin,
                   ::network::mojom::WebSocketMessageType type,
                   uint64_t data_length) override;
  void OnDropChannel(bool was_clean,
                     uint16_t code,
                     const std::string& reason) override;
  void OnClosingHandshake() override;

  // Data pipe read handling.
  void OnReadable(MojoResult result, const mojo::HandleSignalsState& state);
  void TryDeliverMessage();
  void OnRecvTimeout(RecvWaiter* target);

  mojo::Remote<::network::mojom::WebSocket> socket_;
  mojo::Receiver<::network::mojom::WebSocketClient> client_receiver_{this};
  mojo::Receiver<::network::mojom::WebSocketHandshakeClient>
      handshake_receiver_{this};

  mojo::ScopedDataPipeConsumerHandle readable_;
  mojo::ScopedDataPipeProducerHandle writable_;
  mojo::SimpleWatcher read_watcher_;

  bool connected_ = false;
  bool closed_ = false;

  // Incoming frame state.
  uint64_t pending_read_bytes_ = 0;
  std::string pending_message_;

  // Buffered received messages.
  std::vector<std::string> recv_buffer_;

  // Pending recv waiters.
  std::vector<std::unique_ptr<RecvWaiter>> recv_waiters_;

  base::OnceCallback<void(bool)> connect_callback_;
};

}  // namespace clawser::browser

#endif  // CLAWSER_BROWSER_NET_WEBSOCKET_H_
