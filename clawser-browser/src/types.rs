use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A captured HTTP request + response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub method: String,
    pub url: String,
    pub request_headers: Option<serde_json::Value>,
    pub request_body: Option<String>,
    pub status: u16,
    pub response_headers: Option<serde_json::Value>,
    pub response_body: String,
}

/// A captured WebSocket connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedWs {
    pub url: String,
    pub id: u32,
    pub messages: Vec<WsMessage>,
    #[serde(default)]
    pub stack: Option<String>,
}

/// A WebSocket message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub dir: String,
    pub data: String,
    pub ts: u64,
}

/// HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub url: String,
}

/// A browser cookie.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub secure: bool,
    #[serde(default)]
    pub httponly: bool,
}
