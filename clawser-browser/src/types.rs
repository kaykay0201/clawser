use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Deserialize u64 from either a JSON number or a string like "12345".
fn deserialize_u64_flexible<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de;
    let v = serde_json::Value::deserialize(deserializer)?;
    match &v {
        serde_json::Value::Number(n) => n.as_u64().ok_or_else(|| {
            de::Error::custom(format!("number out of u64 range: {}", n))
        }),
        serde_json::Value::String(s) => {
            s.parse::<u64>().map_err(de::Error::custom)
        }
        _ => Err(de::Error::custom(format!("expected number or string, got {}", v))),
    }
}

fn serialize_u64_as_string<S>(val: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&val.to_string())
}

/// Deterministic seed for antidetect profile. Same seed = same fingerprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seed {
    #[serde(deserialize_with = "deserialize_u64_flexible", serialize_with = "serialize_u64_as_string")]
    pub hw_seed: u64,
    #[serde(deserialize_with = "deserialize_u64_flexible", serialize_with = "serialize_u64_as_string")]
    pub canvas_seed: u64,
    #[serde(deserialize_with = "deserialize_u64_flexible", serialize_with = "serialize_u64_as_string")]
    pub webgl_seed: u64,
    #[serde(deserialize_with = "deserialize_u64_flexible", serialize_with = "serialize_u64_as_string")]
    pub audio_seed: u64,
    #[serde(deserialize_with = "deserialize_u64_flexible", serialize_with = "serialize_u64_as_string")]
    pub client_rects_seed: u64,
}

/// A captured API call — either HTTP or WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CapturedCall {
    #[serde(rename = "http")]
    Http {
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<String>,
        #[serde(default)]
        caller_fn_id: String,
    },
    #[serde(rename = "websocket")]
    WebSocket {
        url: String,
        #[serde(default)]
        messages: Vec<WsEvent>,
        #[serde(default)]
        caller_fn_id: String,
    },
}

/// A WebSocket event (sent or received message).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    pub direction: String,
    pub data: String,
    #[serde(default)]
    pub timestamp: u64,
}

/// HTTP response from the browser's network stack.
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
