use std::io;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub(crate) type CdpSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Connect to a CDP WebSocket endpoint.
pub(crate) async fn connect_cdp(ws_url: &str) -> io::Result<CdpSocket> {
    let (socket, _) = connect_async(ws_url)
        .await
        .map_err(|e| io::Error::other(format!("CDP WebSocket connect failed: {}", e)))?;
    Ok(socket)
}

/// Send a CDP method and return the message id.
pub(crate) async fn send_cdp(ws: &mut CdpSocket, method: &str, params: Value) -> io::Result<u64> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let msg = serde_json::json!({
        "id": id,
        "method": method,
        "params": params,
    });
    let text = serde_json::to_string(&msg)
        .map_err(|e| io::Error::other(format!("serialize failed: {}", e)))?;
    ws.send(Message::Text(text.into()))
        .await
        .map_err(|e| io::Error::other(format!("ws send failed: {}", e)))?;
    Ok(id)
}

/// Read CDP responses until we get one matching the given id.
pub(crate) async fn recv_cdp(ws: &mut CdpSocket, expected_id: u64) -> io::Result<Value> {
    loop {
        let msg = ws
            .next()
            .await
            .ok_or_else(|| io::Error::new(io::ErrorKind::ConnectionAborted, "CDP WebSocket closed"))?
            .map_err(|e| io::Error::other(format!("ws read failed: {}", e)))?;

        match msg {
            Message::Text(text) => {
                let parsed: Value = serde_json::from_str(&text)
                    .map_err(|e| io::Error::other(format!("parse failed: {}", e)))?;
                if let Some(id) = parsed.get("id").and_then(|v| v.as_u64()) {
                    if id == expected_id {
                        if let Some(err) = parsed.get("error") {
                            let msg = err
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown CDP error");
                            return Err(io::Error::other(format!("CDP error: {}", msg)));
                        }
                        return Ok(parsed);
                    }
                }
                // Event — ignore, keep reading
            }
            Message::Close(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "CDP WebSocket closed",
                ));
            }
            _ => {} // ping/pong/binary
        }
    }
}

/// Send a CDP method and wait for the response.
pub(crate) async fn call_cdp(ws: &mut CdpSocket, method: &str, params: Value) -> io::Result<Value> {
    let id = send_cdp(ws, method, params).await?;
    recv_cdp(ws, id).await
}
