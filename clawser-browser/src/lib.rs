//! # clawser-browser
//!
//! Antidetect headless browser with API payload capture.
//!
//! ## Quick Start
//!
//! ```no_run
//! use clawser_browser::Browser;
//!
//! let (browser, seed) = Browser::new()?;
//! let page = browser.navigate("https://target.com")?;
//!
//! // Watch HTTP — blocks until endpoint fires, returns closures + response
//! let (re_fetch, gen_payload, resp) = page.watch("/api/v1/setup", 30_000)?;
//! println!("Captured response: {} {}", resp.status, resp.url);
//! let fresh = gen_payload.call()?;  // re-invoke JS caller → new payload
//! let replayed = re_fetch.call()?;  // replay exact HTTP request → new response
//!
//! // Watch WebSocket — blocks until WS created, returns closure + URL + connection
//! let (regen_url, url, ws) = page.watch_websock("/api/lobby", 30_000)?;
//! println!("Captured WS URL: {}", url);
//! ws.send("hello")?;
//! let msg = ws.recv(5000)?;
//! let new_url = regen_url.call()?;  // re-invoke JS caller → new WS URL
//!
//! browser.shutdown()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod process;
mod protocol;
pub mod types;

pub use types::{CapturedCall, Cookie, Response, Seed, WsEvent};

use std::io::{self, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::Mutex;

/// A browser instance = 1 process = 1 antidetect profile.
pub struct Browser {
    child: Child,
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<BufReader<ChildStdout>>,
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
/// Fully antidetect — uses same TLS/H2 fingerprint, zero JS injection.
pub struct WsConnection<'b> {
    browser: &'b Browser,
    ws_id: String,
}

impl Browser {
    /// Create a browser with a random antidetect profile.
    pub fn new() -> io::Result<(Browser, Seed)> {
        Self::create(None, &[])
    }

    /// Create a browser with a deterministic profile from a seed.
    pub fn from_seed(seed: &Seed) -> io::Result<Browser> {
        let (browser, _) = Self::create(Some(seed), &[])?;
        Ok(browser)
    }

    /// Internal: spawn process, send init, parse seed response.
    fn create(seed: Option<&Seed>, watch: &[String]) -> io::Result<(Browser, Seed)> {
        let (child, mut stdin, mut stdout) = process::spawn_browser()?;
        protocol::send_init(&mut stdin, seed, watch)?;
        let response = protocol::read_response(&mut stdout)?;
        let seed_val = response
            .get("seed")
            .ok_or_else(|| io::Error::other("init response missing 'seed'"))?;
        let seed: Seed = serde_json::from_value(seed_val.clone())
            .map_err(|e| io::Error::other(format!("failed to parse seed: {}", e)))?;

        let browser = Browser {
            child,
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(stdout),
        };
        Ok((browser, seed))
    }

    /// Send a command and read the response.
    fn command(&self, cmd: &str, params: serde_json::Value) -> io::Result<serde_json::Value> {
        let mut stdin = self.stdin.lock().map_err(|_| io::Error::other("stdin lock poisoned"))?;
        let _id = protocol::send_command(&mut stdin, cmd, params)?;
        drop(stdin);
        let mut stdout = self.stdout.lock().map_err(|_| io::Error::other("stdout lock poisoned"))?;
        protocol::read_response(&mut stdout)
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
        if let Ok(mut stdin) = self.stdin.lock() {
            let _ = writeln!(stdin, r#"{{"id":0,"cmd":"shutdown"}}"#);
            let _ = stdin.flush();
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
    ///
    /// Returns `(re_fetch, generate_payload, response)`:
    /// - `re_fetch` — replays the exact HTTP request via C++ network stack
    /// - `generate_payload` — re-invokes the JS caller for a fresh payload
    /// - `response` — the HTTP response from the original request
    pub fn watch(
        &self,
        endpoint: &str,
        timeout_ms: u32,
    ) -> io::Result<(ReFetch<'b>, GeneratePayload<'b>, Response)> {
        // Register the watch.
        let watch_resp = self.browser.command(
            "watch",
            serde_json::json!({"endpoint": endpoint}),
        )?;
        let watch_id = watch_resp
            .get("watch_id")
            .and_then(|v| v.as_str())
            .unwrap_or("w0")
            .to_string();

        // Block until endpoint fires.
        let wait_resp = self.browser.command(
            "wait",
            serde_json::json!({"watch_id": watch_id, "timeout_ms": timeout_ms}),
        )?;

        let captured = wait_resp
            .get("captured")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        // Extract request params for ReFetch.
        let method = captured.get("method").and_then(|v| v.as_str()).unwrap_or("GET").to_string();
        let url = captured.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let headers = captured.get("headers").cloned().unwrap_or(serde_json::json!({}));
        let body = captured.get("body").and_then(|v| v.as_str()).map(String::from);

        // Extract response (from the response field, populated by hook's .then()).
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
    ///
    /// Returns `(regenerate_url, captured_url, ws_connection)`:
    /// - `regenerate_url` — re-invokes the JS that generated the WS URL
    /// - `captured_url` — the WebSocket URL (with tokens/params)
    /// - `ws_connection` — a live WebSocket connection to that URL
    pub fn watch_websock(
        &self,
        endpoint: &str,
        timeout_ms: u32,
    ) -> io::Result<(RegenerateUrl<'b>, String, WsConnection<'b>)> {
        // Register the watch.
        let watch_resp = self.browser.command(
            "watch",
            serde_json::json!({"endpoint": endpoint}),
        )?;
        let watch_id = watch_resp
            .get("watch_id")
            .and_then(|v| v.as_str())
            .unwrap_or("w0")
            .to_string();

        // Block until WS endpoint fires.
        let wait_resp = self.browser.command(
            "wait",
            serde_json::json!({"watch_id": watch_id, "timeout_ms": timeout_ms}),
        )?;

        let captured = wait_resp
            .get("captured")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let url = captured.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();

        // Open our own WS connection to the same URL via C++ network stack.
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
    /// Replay the exact captured HTTP request. Returns a fresh response.
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
    /// Re-invoke the JS caller function. Returns new captured request data.
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
    /// Re-invoke the JS caller. Returns the new WebSocket URL with fresh tokens.
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
