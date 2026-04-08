//! # clawser-browser
//!
//! Antidetect headless/headful browser with API payload capture.
//!
//! ## Quick Start
//!
//! ```no_run
//! use clawser_browser::Browser;
//!
//! let (browser, seed) = Browser::new()?;  // headless, random profile
//! let page = browser.navigate("https://target.com")?;
//!
//! // Watch HTTP — blocks until endpoint fires
//! let (re_fetch, gen_payload, resp) = page.watch("/api/v1/setup", 30_000)?;
//! let fresh = gen_payload.call()?;
//! let replayed = re_fetch.call()?;
//!
//! browser.shutdown()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Headful Mode
//!
//! ```no_run
//! use clawser_browser::Browser;
//!
//! let (browser, seed) = Browser::builder()
//!     .headful()
//!     .build()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod process;
mod protocol;
pub mod types;

pub use types::{CapturedCall, Cookie, Response, Seed, WsEvent};

use std::io::{self, BufReader, Write};
use std::net::TcpStream;
use std::process::Child;
use std::sync::Mutex;

/// Builder for creating a browser instance.
pub struct BrowserBuilder {
    headless: bool,
    seed: Option<Seed>,
    watch: Vec<String>,
}

/// A browser instance = 1 process = 1 antidetect profile.
pub struct Browser {
    child: Child,
    writer: Mutex<TcpStream>,
    reader: Mutex<BufReader<TcpStream>>,
}

/// A loaded page within the browser.
pub struct Page<'b> {
    browser: &'b Browser,
    page_id: String,
}

/// Re-sends the exact captured HTTP request via the browser's C++ network stack.
pub struct ReFetch<'b> {
    browser: &'b Browser,
    method: String,
    url: String,
    headers: serde_json::Value,
    body: Option<String>,
}

/// Re-invokes the JS caller function to generate a fresh payload.
pub struct GeneratePayload<'b> {
    browser: &'b Browser,
    watch_id: String,
}

/// Re-invokes the JS caller to generate a fresh WebSocket URL.
pub struct RegenerateUrl<'b> {
    browser: &'b Browser,
    watch_id: String,
}

/// Builder for HTTP requests via the browser's network stack.
pub struct RequestBuilder<'b> {
    browser: &'b Browser,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    timeout_ms: Option<u32>,
}

/// A WebSocket connection via the browser's C++ network stack (Mojo).
pub struct WsConnection<'b> {
    browser: &'b Browser,
    ws_id: String,
}

// --- BrowserBuilder ---

impl BrowserBuilder {
    /// Run with a visible browser window (default is headless).
    pub fn headful(mut self) -> Self {
        self.headless = false;
        self
    }

    /// Run headless (no window). This is the default.
    pub fn headless(mut self) -> Self {
        self.headless = true;
        self
    }

    /// Use a specific seed for deterministic fingerprint profile.
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Pre-register watch endpoints before navigation.
    pub fn watch(mut self, endpoints: &[&str]) -> Self {
        self.watch = endpoints.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Spawn the browser process and wait for it to be ready.
    pub fn build(self) -> io::Result<(Browser, Seed)> {
        let (child, reader, writer, seed) =
            process::spawn_browser(self.headless, self.seed.as_ref(), &self.watch)?;
        let browser = Browser {
            child,
            writer: Mutex::new(writer),
            reader: Mutex::new(reader),
        };
        Ok((browser, seed))
    }
}

// --- Browser ---

impl Browser {
    /// Create a headless browser with a random antidetect profile.
    pub fn new() -> io::Result<(Browser, Seed)> {
        Self::builder().build()
    }

    /// Create a builder for fine-grained control.
    pub fn builder() -> BrowserBuilder {
        BrowserBuilder {
            headless: true,
            seed: None,
            watch: Vec::new(),
        }
    }

    /// Send a command and read the response.
    fn command(&self, cmd: &str, params: serde_json::Value) -> io::Result<serde_json::Value> {
        let mut writer = self.writer.lock().map_err(|_| io::Error::other("writer lock poisoned"))?;
        let _id = protocol::send_command(&mut *writer, cmd, params)?;
        drop(writer);
        let mut reader = self.reader.lock().map_err(|_| io::Error::other("reader lock poisoned"))?;
        protocol::read_response(&mut *reader)
    }

    /// Navigate to a URL. Returns a Page handle.
    pub fn navigate(&self, url: &str) -> io::Result<Page<'_>> {
        let resp = self.command("navigate", serde_json::json!({"url": url}))?;
        let page_id = resp
            .get("page_id")
            .and_then(|v| v.as_str())
            .unwrap_or("p0")
            .to_string();
        Ok(Page {
            browser: self,
            page_id,
        })
    }

    /// Create a fetch request builder.
    pub fn fetch(&self, method: &str, url: &str) -> RequestBuilder<'_> {
        RequestBuilder {
            browser: self,
            method: method.to_string(),
            url: url.to_string(),
            headers: Vec::new(),
            body: None,
            timeout_ms: None,
        }
    }

    /// Open a WebSocket via the browser's C++ network stack.
    pub fn websocket(&self, url: &str) -> io::Result<WsConnection<'_>> {
        let resp = self.command("websocket", serde_json::json!({"url": url}))?;
        let ws_id = resp
            .get("ws_id")
            .and_then(|v| v.as_str())
            .unwrap_or("ws0")
            .to_string();
        Ok(WsConnection {
            browser: self,
            ws_id,
        })
    }

    /// Get cookies. Pass empty string for all cookies, or a URL for site-specific.
    pub fn cookies(&self, url: &str) -> io::Result<Vec<types::Cookie>> {
        let resp = self.command("cookies", serde_json::json!({"url": url}))?;
        let cookies: Vec<types::Cookie> = resp
            .get("cookies")
            .cloned()
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .unwrap_or_default();
        Ok(cookies)
    }

    /// Shut down the browser process cleanly.
    pub fn shutdown(mut self) -> io::Result<()> {
        let _ = self.command("shutdown", serde_json::json!({}));
        self.child.wait()?;
        Ok(())
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writeln!(writer, r#"{{"id":0,"cmd":"shutdown"}}"#);
            let _ = writer.flush();
        }
        let _ = self.child.wait();
    }
}

impl<'b> Page<'b> {
    /// Execute JavaScript in the page's main world.
    pub fn js(&self, code: &str) -> io::Result<String> {
        let resp = self.browser.command(
            "js",
            serde_json::json!({"page_id": self.page_id, "code": code}),
        )?;
        Ok(resp
            .get("result")
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .unwrap_or_default())
    }

    /// Watch an HTTP endpoint. Blocks until the endpoint fires.
    pub fn watch(
        &self,
        endpoint: &str,
        timeout_ms: u32,
    ) -> io::Result<(ReFetch<'b>, GeneratePayload<'b>, Response)> {
        let watch_resp = self.browser.command(
            "watch",
            serde_json::json!({"endpoint": endpoint}),
        )?;
        let watch_id = watch_resp
            .get("watch_id")
            .and_then(|v| v.as_str())
            .unwrap_or("w0")
            .to_string();

        let wait_resp = self.browser.command(
            "wait",
            serde_json::json!({"watch_id": watch_id, "timeout_ms": timeout_ms}),
        )?;

        let captured = wait_resp
            .get("captured")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        let method = captured.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
        let url = captured.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let headers = captured.get("headers").cloned().unwrap_or(serde_json::json!({}));
        let body = captured.get("body").and_then(|v| v.as_str()).map(String::from);

        let resp_data = captured.get("response").cloned().unwrap_or(serde_json::Value::Null);
        let response = Response {
            status: resp_data.get("status").and_then(|v| v.as_u64()).unwrap_or(0) as u16,
            headers: resp_data
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default(),
            body: resp_data
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .as_bytes()
                .to_vec(),
            url: resp_data
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or(&url)
                .to_string(),
        };

        Ok((
            ReFetch {
                browser: self.browser,
                method,
                url: url.clone(),
                headers,
                body,
            },
            GeneratePayload {
                browser: self.browser,
                watch_id,
            },
            response,
        ))
    }

    /// Watch a WebSocket endpoint. Blocks until a WS connection is created.
    pub fn watch_websock(
        &self,
        endpoint: &str,
        timeout_ms: u32,
    ) -> io::Result<(RegenerateUrl<'b>, String, WsConnection<'b>)> {
        let watch_resp = self.browser.command(
            "watch",
            serde_json::json!({"endpoint": endpoint}),
        )?;
        let watch_id = watch_resp
            .get("watch_id")
            .and_then(|v| v.as_str())
            .unwrap_or("w0")
            .to_string();

        let wait_resp = self.browser.command(
            "wait",
            serde_json::json!({"watch_id": watch_id, "timeout_ms": timeout_ms}),
        )?;

        let captured = wait_resp
            .get("captured")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let url = captured.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();

        let ws = self.browser.websocket(&url)?;

        Ok((
            RegenerateUrl {
                browser: self.browser,
                watch_id,
            },
            url,
            ws,
        ))
    }
}

// --- Closure types ---

impl<'b> ReFetch<'b> {
    pub fn call(&self) -> io::Result<Response> {
        let mut params = serde_json::json!({
            "method": self.method,
            "url": self.url,
            "headers": self.headers,
        });
        if let Some(ref body) = self.body {
            params["body"] = serde_json::Value::from(body.as_str());
        }
        let resp = self.browser.command("fetch", params)?;
        Ok(Response {
            status: resp.get("status").and_then(|v| v.as_u64()).unwrap_or(0) as u16,
            headers: resp
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default(),
            body: resp.get("body").and_then(|v| v.as_str()).unwrap_or("").as_bytes().to_vec(),
            url: resp.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    }
}

impl<'b> GeneratePayload<'b> {
    pub fn call(&self) -> io::Result<CapturedCall> {
        let resp = self.browser.command(
            "replay",
            serde_json::json!({"watch_id": self.watch_id}),
        )?;
        let captured: CapturedCall = serde_json::from_value(
            resp.get("captured").cloned().unwrap_or(serde_json::Value::Null),
        )
        .map_err(|e| io::Error::other(format!("failed to parse replay: {}", e)))?;
        Ok(captured)
    }
}

impl<'b> RegenerateUrl<'b> {
    pub fn call(&self) -> io::Result<String> {
        let resp = self.browser.command(
            "replay",
            serde_json::json!({"watch_id": self.watch_id}),
        )?;
        let captured = resp.get("captured").cloned().unwrap_or(serde_json::Value::Null);
        Ok(captured.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string())
    }
}

// --- RequestBuilder ---

impl<'b> RequestBuilder<'b> {
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn body(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.body = Some(data.into());
        self
    }

    pub fn timeout_ms(mut self, ms: u32) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    pub fn send(self) -> io::Result<Response> {
        let mut params = serde_json::json!({
            "method": self.method,
            "url": self.url,
        });
        if !self.headers.is_empty() {
            let headers: serde_json::Map<String, serde_json::Value> = self
                .headers
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::from(v)))
                .collect();
            params["headers"] = serde_json::Value::Object(headers);
        }
        if let Some(body) = self.body {
            params["body"] = serde_json::Value::from(String::from_utf8_lossy(&body).to_string());
        }
        if let Some(ms) = self.timeout_ms {
            params["timeout_ms"] = serde_json::Value::from(ms);
        }
        let resp = self.browser.command("fetch", params)?;
        Ok(Response {
            status: resp.get("status").and_then(|v| v.as_u64()).unwrap_or(0) as u16,
            headers: resp
                .get("headers")
                .and_then(|v| v.as_object())
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default(),
            body: resp.get("body").and_then(|v| v.as_str()).unwrap_or("").as_bytes().to_vec(),
            url: resp.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        })
    }
}

// --- WsConnection ---

impl<'b> WsConnection<'b> {
    pub fn send(&self, data: impl AsRef<str>) -> io::Result<()> {
        self.browser.command(
            "ws_send",
            serde_json::json!({"ws_id": self.ws_id, "data": data.as_ref()}),
        )?;
        Ok(())
    }

    pub fn recv(&self, timeout_ms: u32) -> io::Result<String> {
        let resp = self.browser.command(
            "ws_recv",
            serde_json::json!({"ws_id": self.ws_id, "timeout_ms": timeout_ms}),
        )?;
        Ok(resp.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string())
    }

    pub fn close(self) -> io::Result<()> {
        self.browser.command("ws_close", serde_json::json!({"ws_id": self.ws_id}))?;
        Ok(())
    }
}
