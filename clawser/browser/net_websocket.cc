// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "clawser/browser/net_websocket.h"

#include "net/traffic_annotation/network_traffic_annotation.h"
#include "services/network/public/cpp/resource_request.h"
#include "url/origin.h"

namespace clawser::browser {

namespace {

net::NetworkTrafficAnnotationTag GetTrafficAnnotation() {
  return net::DefineNetworkTrafficAnnotation("clawser_websocket", R"(
    semantics {
      sender: "Clawser Browser"
      description: "WebSocket connection for antidetect browser automation."
      trigger: "User-initiated via Rust API."
      data: "Application-level WebSocket messages."
      destination: OTHER
    }
    policy {
      cookies_allowed: YES
      cookies_store: "user profile"
    })");
}

}  // namespace

NetWebSocket::NetWebSocket()
    : read_watcher_(FROM_HERE,
                    mojo::SimpleWatcher::ArmingPolicy::AUTOMATIC) {}

NetWebSocket::~NetWebSocket() {
  Close();
}

void NetWebSocket::Connect(
    const GURL& url,
    network::mojom::NetworkContext* network_context,
    const url::Origin& origin,
    base::OnceCallback<void(bool)> cb) {
  connect_callback_ = std::move(cb);

  std::vector<network::mojom::HttpHeaderPtr> headers;
  network_context->CreateWebSocket(
      url,
      /*requested_protocols=*/{},
      net::SiteForCookies::FromOrigin(origin),
      net::StorageAccessApiStatus::kNone,
      net::IsolationInfo::CreateForInternalRequest(origin),
      std::move(headers),
      /*process_id=*/network::mojom::kBrowserProcessId,
      origin,
      /*options=*/0,
      net::MutableNetworkTrafficAnnotationTag(GetTrafficAnnotation()),
      handshake_receiver_.BindNewPipeAndPassRemote(),
      /*url_loader_network_observer=*/mojo::NullRemote(),
      /*auth_handler=*/mojo::NullRemote(),
      /*header_client=*/mojo::NullRemote(),
      /*throttling_profile_id=*/std::nullopt);
}

void NetWebSocket::Send(const std::string& data,
                        base::OnceCallback<void(bool)> cb) {
  if (!connected_ || closed_ || !writable_.is_valid()) {
    std::move(cb).Run(false);
    return;
  }

  base::span<const uint8_t> bytes = base::as_byte_span(data);
  size_t actually_written = 0;
  MojoResult result = writable_->WriteData(bytes, MOJO_WRITE_DATA_FLAG_NONE,
                                           actually_written);
  if (result != MOJO_RESULT_OK || actually_written != data.size()) {
    std::move(cb).Run(false);
    return;
  }

  socket_->SendMessage(network::mojom::WebSocketMessageType::TEXT,
                       data.size());
  std::move(cb).Run(true);
}

void NetWebSocket::Recv(uint32_t timeout_ms,
                        base::OnceCallback<void(std::string)> cb) {
  // Deliver immediately if buffer has data.
  if (!recv_buffer_.empty()) {
    std::string msg = std::move(recv_buffer_.front());
    recv_buffer_.erase(recv_buffer_.begin());
    std::move(cb).Run(std::move(msg));
    return;
  }

  if (closed_) {
    std::move(cb).Run("");
    return;
  }

  // Register a waiter with its pointer captured in the timeout callback.
  auto waiter = std::make_unique<RecvWaiter>();
  waiter->callback = std::move(cb);
  RecvWaiter* raw = waiter.get();
  waiter->timeout.Start(
      FROM_HERE, base::Milliseconds(timeout_ms),
      base::BindOnce(&NetWebSocket::OnRecvTimeout,
                     base::Unretained(this), raw));
  recv_waiters_.push_back(std::move(waiter));
}

void NetWebSocket::Close() {
  if (connected_ && !closed_ && socket_.is_bound()) {
    socket_->StartClosingHandshake(1000, "");
  }
  closed_ = true;
  read_watcher_.Cancel();
  readable_.reset();
  writable_.reset();
  socket_.reset();
  client_receiver_.reset();
  handshake_receiver_.reset();

  // Resolve any pending waiters.
  for (auto& w : recv_waiters_) {
    w->timeout.Stop();
    std::move(w->callback).Run("");
  }
  recv_waiters_.clear();
}

// --- WebSocketHandshakeClient ---

void NetWebSocket::OnOpeningHandshakeStarted(
    network::mojom::WebSocketHandshakeRequestPtr request) {}

void NetWebSocket::OnFailure(const std::string& message,
                             int32_t net_error,
                             int32_t response_code) {
  closed_ = true;
  if (connect_callback_)
    std::move(connect_callback_).Run(false);
}

void NetWebSocket::OnConnectionEstablished(
    mojo::PendingRemote<network::mojom::WebSocket> socket,
    mojo::PendingReceiver<network::mojom::WebSocketClient> client_receiver,
    network::mojom::WebSocketHandshakeResponsePtr response,
    mojo::ScopedDataPipeConsumerHandle readable,
    mojo::ScopedDataPipeProducerHandle writable) {
  socket_.Bind(std::move(socket));
  client_receiver_.Bind(std::move(client_receiver));
  readable_ = std::move(readable);
  writable_ = std::move(writable);
  connected_ = true;

  // Watch the readable pipe for incoming data.
  read_watcher_.Watch(
      readable_.get(),
      MOJO_HANDLE_SIGNAL_READABLE | MOJO_HANDLE_SIGNAL_PEER_CLOSED,
      base::BindRepeating(&NetWebSocket::OnReadable,
                          base::Unretained(this)));

  // Tell network service we're ready to receive.
  socket_->StartReceiving();

  if (connect_callback_)
    std::move(connect_callback_).Run(true);
}

// --- WebSocketClient ---

void NetWebSocket::OnDataFrame(bool fin,
                               network::mojom::WebSocketMessageType type,
                               uint64_t data_length) {
  pending_read_bytes_ += data_length;
  if (fin) {
    // Full message pending — try to read it.
    TryDeliverMessage();
  }
}

void NetWebSocket::OnDropChannel(bool was_clean,
                                 uint16_t code,
                                 const std::string& reason) {
  closed_ = true;
  // Resolve pending waiters.
  for (auto& w : recv_waiters_) {
    w->timeout.Stop();
    std::move(w->callback).Run("");
  }
  recv_waiters_.clear();
}

void NetWebSocket::OnClosingHandshake() {}

// --- Data pipe reading ---

void NetWebSocket::OnReadable(MojoResult result,
                              const mojo::HandleSignalsState& state) {
  if (result != MOJO_RESULT_OK) {
    if (state.peer_closed())
      closed_ = true;
    return;
  }
  TryDeliverMessage();
}

void NetWebSocket::TryDeliverMessage() {
  if (pending_read_bytes_ == 0 || !readable_.is_valid())
    return;

  // Read available data from the pipe.
  std::vector<uint8_t> buffer(pending_read_bytes_);
  size_t actually_read = 0;
  MojoResult result = readable_->ReadData(
      MOJO_READ_DATA_FLAG_NONE,
      base::span<uint8_t>(buffer.data(), buffer.size()),
      actually_read);

  if (result != MOJO_RESULT_OK)
    return;

  pending_message_.append(reinterpret_cast<const char*>(buffer.data()),
                          actually_read);
  pending_read_bytes_ -= actually_read;

  if (pending_read_bytes_ == 0) {
    // Complete message received.
    std::string msg = std::move(pending_message_);
    pending_message_.clear();

    // Deliver to a waiter or buffer it.
    if (!recv_waiters_.empty()) {
      auto waiter = std::move(recv_waiters_.front());
      recv_waiters_.erase(recv_waiters_.begin());
      waiter->timeout.Stop();
      std::move(waiter->callback).Run(std::move(msg));
    } else {
      recv_buffer_.push_back(std::move(msg));
    }
  }
}

void NetWebSocket::OnRecvTimeout(RecvWaiter* target) {
  for (auto it = recv_waiters_.begin(); it != recv_waiters_.end(); ++it) {
    if (it->get() == target) {
      auto waiter = std::move(*it);
      recv_waiters_.erase(it);
      waiter->timeout.Stop();
      std::move(waiter->callback).Run("");
      return;
    }
  }
}

}  // namespace clawser::browser
