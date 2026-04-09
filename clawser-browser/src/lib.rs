//! # clawser-browser
//!
//! Async antidetect browser powered by a patched Chromium + CDP.
//! Native tokio support — all methods are `async`.
//!
//! ```no_run
//! use clawser_browser::Browser;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let browser = Browser::builder()
//!         .headful()
//!         .random()
//!         .build().await?;
//!     let page = browser.navigate("https://example.com").await?;
//!     page.human_idle(2000).await?;
//!     let title = page.js("document.title").await?;
//!     println!("{}", title);
//!     browser.shutdown().await?;
//!     Ok(())
//! }
//! ```

mod process;
pub mod profiles;
mod protocol;
pub mod types;

pub use profiles::HwProfile;
pub use types::{Cookie, Response};

use std::io;
use std::time::Duration;

use tokio::process::Child;
use tokio::sync::Mutex;

/// Builder for creating a browser instance.
pub struct BrowserBuilder {
    headless: bool,
    config_path: Option<String>,
    profile_index: Option<usize>,
    seed_index: Option<u64>,
}

/// A browser instance = 1 chrome.exe process + CDP connection.
pub struct Browser {
    child: Mutex<Child>,
    #[allow(dead_code)]
    cdp_port: u16,
    ws: Mutex<protocol::CdpSocket>,
}

/// A loaded page within the browser.
pub struct Page<'b> {
    browser: &'b Browser,
    _target_id: String,
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

    /// Path to clawser config JSON (antidetect profile).
    pub fn config(mut self, path: &str) -> Self {
        self.config_path = Some(path.to_string());
        self
    }

    /// Use a random profile from the 100 built-in device profiles.
    pub fn random(mut self) -> Self {
        self.profile_index = Some(profiles::random_profile_index());
        self.seed_index = Some(profiles::random_seed_index());
        self
    }

    /// Use a specific profile (0..99) with a specific seed.
    /// Same (profile, seed) always produces the same fingerprint.
    pub fn profile(mut self, profile_index: usize, seed_index: u64) -> Self {
        self.profile_index = Some(profile_index);
        self.seed_index = Some(seed_index);
        self
    }

    /// Spawn chrome.exe and connect via CDP.
    pub async fn build(self) -> io::Result<Browser> {
        // If profile specified, generate config to temp file
        let generated_path;
        let config_path = if let (Some(pi), Some(si)) = (self.profile_index, self.seed_index) {
            generated_path = profiles::write_config_file(pi, si)?;
            generated_path.as_str()
        } else {
            generated_path = String::new();
            self.config_path.as_deref().unwrap_or("")
        };

        let profile_id = match (self.profile_index, self.seed_index) {
            (Some(pi), Some(si)) => Some(format!("p{}-s{}", pi, si)),
            _ => None,
        };

        let cdp_port = process::pick_free_port()?;
        let child = process::spawn_chrome(
            self.headless,
            cdp_port,
            config_path,
            profile_id.as_deref(),
        )?;

        // Wait for CDP to be ready
        process::wait_for_cdp(cdp_port, Duration::from_secs(30)).await?;

        // Connect to the first page's WebSocket
        let ws_url = process::get_page_ws_url(cdp_port).await?;
        let ws = protocol::connect_cdp(&ws_url).await?;

        Ok(Browser {
            child: Mutex::new(child),
            cdp_port,
            ws: Mutex::new(ws),
        })
    }
}

// --- Browser ---

impl Browser {
    /// Create a headless browser with default config.
    pub async fn new() -> io::Result<Browser> {
        Self::builder().build().await
    }

    /// Create a builder for fine-grained control.
    pub fn builder() -> BrowserBuilder {
        BrowserBuilder {
            headless: true,
            config_path: None,
            profile_index: None,
            seed_index: None,
        }
    }

    /// Navigate the current page to a URL.
    pub async fn navigate(&self, url: &str) -> io::Result<Page<'_>> {
        {
            let mut ws = self.ws.lock().await;
            protocol::call_cdp(&mut ws, "Page.enable", serde_json::json!({})).await?;
            let resp = protocol::call_cdp(
                &mut ws,
                "Page.navigate",
                serde_json::json!({"url": url}),
            )
            .await?;

            let _target_id = resp
                .get("result")
                .and_then(|r| r.get("frameId"))
                .and_then(|v| v.as_str())
                .unwrap_or("main")
                .to_string();
        }

        // Wait for load event
        self.wait_for_load(Duration::from_secs(30)).await?;

        Ok(Page {
            browser: self,
            _target_id: String::new(),
        })
    }

    /// Wait for Page.loadEventFired.
    async fn wait_for_load(&self, timeout: Duration) -> io::Result<()> {
        use futures_util::StreamExt;
        let start = std::time::Instant::now();
        let mut ws = self.ws.lock().await;

        loop {
            if start.elapsed() > timeout {
                return Ok(()); // Timeout not fatal
            }

            let read_result = tokio::time::timeout(
                Duration::from_secs(1),
                ws.next(),
            )
            .await;

            match read_result {
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let method = parsed.get("method").and_then(|v| v.as_str()).unwrap_or("");
                        if method == "Page.loadEventFired"
                            || method == "Page.frameStoppedLoading"
                        {
                            return Ok(());
                        }
                    }
                }
                Err(_) => continue,   // timeout — keep waiting
                Ok(None) => return Ok(()), // stream ended
                _ => {}
            }
        }
    }

    /// Get cookies for a URL (or all if empty).
    pub async fn cookies(&self, url: &str) -> io::Result<Vec<Cookie>> {
        let mut ws = self.ws.lock().await;
        let params = if url.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({"urls": [url]})
        };
        let resp = protocol::call_cdp(&mut ws, "Network.getCookies", params).await?;
        let cookies: Vec<Cookie> = resp
            .get("result")
            .and_then(|r| r.get("cookies"))
            .cloned()
            .map(|v| serde_json::from_value(v).unwrap_or_default())
            .unwrap_or_default();
        Ok(cookies)
    }

    /// Take a screenshot (PNG bytes).
    pub async fn screenshot(&self) -> io::Result<Vec<u8>> {
        let mut ws = self.ws.lock().await;
        let resp = protocol::call_cdp(
            &mut ws,
            "Page.captureScreenshot",
            serde_json::json!({"format": "png"}),
        )
        .await?;
        let data = resp
            .get("result")
            .and_then(|r| r.get("data"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        base64_decode(data)
    }

    /// Shut down the browser process cleanly.
    pub async fn shutdown(self) -> io::Result<()> {
        {
            let mut ws = self.ws.lock().await;
            let _ = protocol::call_cdp(&mut ws, "Browser.close", serde_json::json!({})).await;
        }
        let mut child = self.child.lock().await;
        let _ = child.wait().await;
        Ok(())
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.try_lock() {
            let _ = child.start_kill();
        }
    }
}

// --- Page ---

impl<'b> Page<'b> {
    /// Move mouse along a human-like bezier curve to (x, y).
    pub async fn mouse_move(&self, x: f64, y: f64, steps: u32) -> io::Result<()> {
        let mut ws = self.browser.ws.lock().await;

        let mut rng_buf = [0u8; 16];
        profiles::getrandom(&mut rng_buf);
        let r0 = u64::from_le_bytes(rng_buf[0..8].try_into().unwrap());
        let r1 = u64::from_le_bytes(rng_buf[8..16].try_into().unwrap());

        let start_x = (r0 % 400) as f64 + 100.0;
        let start_y = (r1 % 300) as f64 + 100.0;

        let cp1x = start_x + (x - start_x) * 0.3 + ((r0 >> 16) % 80) as f64 - 40.0;
        let cp1y = start_y + (y - start_y) * 0.1 + ((r0 >> 24) % 60) as f64 - 30.0;
        let cp2x = start_x + (x - start_x) * 0.7 + ((r1 >> 16) % 60) as f64 - 30.0;
        let cp2y = start_y + (y - start_y) * 0.9 + ((r1 >> 24) % 40) as f64 - 20.0;

        let steps = steps.max(5);
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let u = 1.0 - t;
            let px = u * u * u * start_x + 3.0 * u * u * t * cp1x + 3.0 * u * t * t * cp2x + t * t * t * x;
            let py = u * u * u * start_y + 3.0 * u * u * t * cp1y + 3.0 * u * t * t * cp2y + t * t * t * y;

            protocol::call_cdp(&mut ws, "Input.dispatchMouseEvent", serde_json::json!({
                "type": "mouseMoved", "x": px.round(), "y": py.round(),
            }))
            .await?;

            let base_ms = 5 + ((r0.wrapping_add(i as u64 * 7)) % 12);
            tokio::time::sleep(Duration::from_millis(base_ms)).await;
        }
        Ok(())
    }

    /// Click at (x, y) with human-like mouse movement first.
    pub async fn click(&self, x: f64, y: f64) -> io::Result<()> {
        self.mouse_move(x, y, 15).await?;

        let mut ws = self.browser.ws.lock().await;
        protocol::call_cdp(&mut ws, "Input.dispatchMouseEvent", serde_json::json!({
            "type": "mousePressed", "x": x, "y": y, "button": "left", "clickCount": 1,
        }))
        .await?;

        let mut buf = [0u8; 8];
        profiles::getrandom(&mut buf);
        let hold_ms = 50 + (u64::from_le_bytes(buf) % 70);
        tokio::time::sleep(Duration::from_millis(hold_ms)).await;

        protocol::call_cdp(&mut ws, "Input.dispatchMouseEvent", serde_json::json!({
            "type": "mouseReleased", "x": x, "y": y, "button": "left", "clickCount": 1,
        }))
        .await?;
        Ok(())
    }

    /// Type text with human-like timing.
    pub async fn type_text(&self, text: &str) -> io::Result<()> {
        let mut ws = self.browser.ws.lock().await;
        let mut rng_buf = [0u8; 8];

        for ch in text.chars() {
            let key_str = ch.to_string();

            protocol::call_cdp(&mut ws, "Input.dispatchKeyEvent", serde_json::json!({
                "type": "keyDown", "text": key_str, "key": key_str,
            }))
            .await?;

            profiles::getrandom(&mut rng_buf);
            let dwell = 30 + (u64::from_le_bytes(rng_buf) % 50);
            tokio::time::sleep(Duration::from_millis(dwell)).await;

            protocol::call_cdp(&mut ws, "Input.dispatchKeyEvent", serde_json::json!({
                "type": "keyUp", "key": key_str,
            }))
            .await?;

            profiles::getrandom(&mut rng_buf);
            let gap = 40 + (u64::from_le_bytes(rng_buf) % 140);
            tokio::time::sleep(Duration::from_millis(gap)).await;
        }
        Ok(())
    }

    /// Scroll with human-like momentum.
    pub async fn scroll(&self, delta_y: i32) -> io::Result<()> {
        let mut ws = self.browser.ws.lock().await;
        let mut rng_buf = [0u8; 8];
        profiles::getrandom(&mut rng_buf);

        let steps = 5 + (u64::from_le_bytes(rng_buf) % 6) as i32;
        let base_delta = delta_y / steps;
        let mut remaining = delta_y;

        let mx = 400.0 + (u64::from_le_bytes(rng_buf) % 300) as f64;
        let my = 300.0 + ((u64::from_le_bytes(rng_buf) >> 16) % 200) as f64;

        for i in 0..steps {
            let this_delta = if i == steps - 1 {
                remaining
            } else {
                profiles::getrandom(&mut rng_buf);
                let jitter = (u64::from_le_bytes(rng_buf) % 20) as i32 - 10;
                let d = base_delta + jitter;
                remaining -= d;
                d
            };

            protocol::call_cdp(&mut ws, "Input.dispatchMouseEvent", serde_json::json!({
                "type": "mouseWheel", "x": mx, "y": my, "deltaX": 0, "deltaY": this_delta,
            }))
            .await?;

            let delay = 30 + (20 / (i + 1)) as u64;
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
        Ok(())
    }

    /// Simulate idle human presence with random mouse movements.
    pub async fn human_idle(&self, duration_ms: u64) -> io::Result<()> {
        let start = std::time::Instant::now();
        let mut rng_buf = [0u8; 16];

        while (start.elapsed().as_millis() as u64) < duration_ms {
            profiles::getrandom(&mut rng_buf);
            let x = 200.0 + (u64::from_le_bytes(rng_buf[0..8].try_into().unwrap()) % 800) as f64;
            let y = 150.0 + (u64::from_le_bytes(rng_buf[8..16].try_into().unwrap()) % 500) as f64;

            self.mouse_move(x, y, 8).await?;

            profiles::getrandom(&mut rng_buf);
            let pause = 500 + (u64::from_le_bytes(rng_buf[0..8].try_into().unwrap()) % 1500);
            let remaining = duration_ms.saturating_sub(start.elapsed().as_millis() as u64);
            tokio::time::sleep(Duration::from_millis(pause.min(remaining))).await;
        }
        Ok(())
    }

    /// Capture full page as MHTML (HTML + JS + CSS + images — everything).
    /// Save to .mhtml file, opens in Chrome with full fidelity.
    pub async fn capture_mhtml(&self) -> io::Result<Vec<u8>> {
        let mut ws = self.browser.ws.lock().await;
        let resp = protocol::call_cdp(
            &mut ws,
            "Page.captureSnapshot",
            serde_json::json!({"format": "mhtml"}),
        )
        .await?;
        let data = resp
            .get("result")
            .and_then(|r| r.get("data"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        Ok(data.as_bytes().to_vec())
    }

    /// Capture page HTML via DOM API (won't break JS/special chars).
    /// Returns outerHTML including all `<script>` tags intact.
    pub async fn capture_html(&self) -> io::Result<String> {
        let mut ws = self.browser.ws.lock().await;
        let doc = protocol::call_cdp(
            &mut ws,
            "DOM.getDocument",
            serde_json::json!({"depth": 0}),
        )
        .await?;
        let node_id = doc
            .get("result")
            .and_then(|r| r.get("root"))
            .and_then(|r| r.get("nodeId"))
            .and_then(|v| v.as_i64())
            .ok_or_else(|| io::Error::other("DOM.getDocument: no root nodeId"))?;

        let resp = protocol::call_cdp(
            &mut ws,
            "DOM.getOuterHTML",
            serde_json::json!({"nodeId": node_id}),
        )
        .await?;
        Ok(resp
            .get("result")
            .and_then(|r| r.get("outerHTML"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string())
    }

    /// Execute JavaScript and return the result as a string.
    pub async fn js(&self, code: &str) -> io::Result<String> {
        let mut ws = self.browser.ws.lock().await;
        let resp = protocol::call_cdp(
            &mut ws,
            "Runtime.evaluate",
            serde_json::json!({
                "expression": code,
                "returnByValue": true,
            }),
        )
        .await?;

        let result = resp.get("result").and_then(|r| r.get("result"));

        if let Some(result) = result {
            if let Some(exception) = resp.get("result").and_then(|r| r.get("exceptionDetails")) {
                let msg = exception
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("JS exception");
                return Err(io::Error::other(format!("JS error: {}", msg)));
            }

            match result.get("value") {
                Some(serde_json::Value::String(s)) => Ok(s.clone()),
                Some(v) => Ok(v.to_string()),
                None => {
                    let type_str = result.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if type_str == "undefined" {
                        Ok("undefined".to_string())
                    } else {
                        Ok(result
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string())
                    }
                }
            }
        } else {
            Ok(String::new())
        }
    }
}

/// Minimal base64 decoder.
fn base64_decode(input: &str) -> io::Result<Vec<u8>> {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut lookup = [255u8; 256];
    for (i, &c) in TABLE.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }
    let input = input.as_bytes();
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input {
        if b == b'=' || b == b'\n' || b == b'\r' {
            continue;
        }
        let val = lookup[b as usize];
        if val == 255 {
            return Err(io::Error::other(format!("invalid base64 char: {}", b as char)));
        }
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Ok(out)
}
