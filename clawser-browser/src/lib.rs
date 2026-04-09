//! # clawser-browser
//!
//! Antidetect browser automation powered by chromiumoxide CDP.
//! 100 device profiles, human simulation, watch/capture/replay.
//!
//! ```no_run
//! use clawser_browser::Browser;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let browser = Browser::builder()
//!         .headful()
//!         .random()
//!         .watch(&["/api/setup"])
//!         .watch_ws(&["/lobby"])
//!         .build().await?;
//!
//!     let page = browser.navigate("https://example.com").await?;
//!     page.human_idle(2000).await?;
//!     let title = page.js("document.title").await?;
//!     browser.shutdown().await?;
//!     Ok(())
//! }
//! ```

pub mod profiles;
pub mod types;

pub use profiles::HwProfile;
pub use types::{CapturedRequest, CapturedWs, Cookie, WsMessage};

use std::io;
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::browser::{
    Browser as CdpBrowser, BrowserConfig,
};
use chromiumoxide::cdp::browser_protocol::network::EventWebSocketCreated;
use chromiumoxide::cdp::browser_protocol::network::EventWebSocketFrameReceived;
use chromiumoxide::Page as CdpPage;
use futures_util::StreamExt;
use tokio::sync::Mutex;

/// Builder for creating a browser instance.
pub struct BrowserBuilder {
    headless: bool,
    config_path: Option<String>,
    profile_index: Option<usize>,
    seed_index: Option<u64>,
    watch_patterns: Vec<String>,
    watch_ws_patterns: Vec<String>,
}

/// A browser instance wrapping chromiumoxide.
pub struct Browser {
    inner: CdpBrowser,
    _handler: tokio::task::JoinHandle<()>,
    ws_events: Arc<Mutex<Vec<serde_json::Value>>>,
}

/// A loaded page.
pub struct Page {
    inner: CdpPage,
    ws_events: Arc<Mutex<Vec<serde_json::Value>>>,
}

// --- BrowserBuilder ---

impl BrowserBuilder {
    pub fn headful(mut self) -> Self {
        self.headless = false;
        self
    }

    pub fn headless(mut self) -> Self {
        self.headless = true;
        self
    }

    pub fn config(mut self, path: &str) -> Self {
        self.config_path = Some(path.to_string());
        self
    }

    pub fn random(mut self) -> Self {
        self.profile_index = Some(profiles::random_profile_index());
        self.seed_index = Some(profiles::random_seed_index());
        self
    }

    pub fn profile(mut self, profile_index: usize, seed_index: u64) -> Self {
        self.profile_index = Some(profile_index);
        self.seed_index = Some(seed_index);
        self
    }

    pub fn watch(mut self, patterns: &[&str]) -> Self {
        self.watch_patterns = patterns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn watch_ws(mut self, patterns: &[&str]) -> Self {
        self.watch_ws_patterns = patterns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub async fn build(self) -> io::Result<Browser> {
        // Generate config if profile specified
        let config_path = if let (Some(pi), Some(si)) = (self.profile_index, self.seed_index) {
            profiles::write_config_file(pi, si)?
        } else {
            self.config_path.unwrap_or_default()
        };

        let profile_id = match (self.profile_index, self.seed_index) {
            (Some(pi), Some(si)) => format!("p{}-s{}", pi, si),
            _ => String::new(),
        };

        // Find chrome executable
        let chrome_path = find_chrome_exe()?;

        // User data dir — project-scoped
        let user_data_dir = if !profile_id.is_empty() {
            let project_root = std::env::var("CARGO_MANIFEST_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| ".".into()));
            project_root.join(".clawser").join("profiles").join(&profile_id)
        } else {
            std::env::temp_dir().join(format!("clawser-{}", std::process::id()))
        };

        let mut builder = BrowserConfig::builder()
            .chrome_executable(chrome_path)
            .user_data_dir(&user_data_dir)
            .disable_default_args()
            .arg("--no-first-run")
            .arg("--disable-default-apps")
            .arg("--disable-sync")
            .arg("--no-sandbox")
            .arg("--disable-blink-features=AutomationControlled");

        if !config_path.is_empty() {
            builder = builder.arg(("clawser-config", config_path.as_str()));
        }

        if self.headless {
            builder = builder.arg("--headless=new");
        } else {
            builder = builder.with_head();
        }

        let browser_config = builder.build()
            .map_err(|e| io::Error::other(format!("BrowserConfig error: {}", e)))?;

        let (browser, mut handler) = CdpBrowser::launch(browser_config).await
            .map_err(|e| io::Error::other(format!("Browser launch failed: {}", e)))?;

        // Spawn handler
        let handler_handle = tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() { break; }
            }
        });

        let ws_events = Arc::new(Mutex::new(Vec::new()));

        // Inject HTTP watch hooks if patterns specified
        if !self.watch_patterns.is_empty() {
            let patterns_json = serde_json::to_string(&self.watch_patterns).unwrap_or("[]".into());
            let pages = browser.pages().await
                .map_err(|e| io::Error::other(format!("pages failed: {}", e)))?;
            if let Some(page) = pages.first() {
                let hook = format!(r#"
(function() {{
    window.__clawser_patterns = {p};
    window.__clawser_captures = [];
    const _fetch = window.fetch;
    window.fetch = function(input, init) {{
        const url = String(input instanceof Request ? input.url : input);
        const method = init?.method || (input instanceof Request ? input.method : 'GET') || 'GET';
        const match = window.__clawser_patterns.find(p => url.includes(p));
        if (!match) return _fetch.apply(this, arguments);
        const reqBody = init?.body || null;
        const reqHeaders = init?.headers || null;
        return _fetch.apply(this, arguments).then(async resp => {{
            const clone = resp.clone();
            const body = await clone.text().catch(() => '');
            const rh = {{}};
            clone.headers.forEach((v,k) => rh[k] = v);
            window.__clawser_captures.push({{
                pattern: match, method, url,
                request_headers: reqHeaders,
                request_body: typeof reqBody === 'string' ? reqBody : null,
                status: resp.status,
                response_headers: rh,
                response_body: body
            }});
            return resp;
        }});
    }};
    const _open = XMLHttpRequest.prototype.open;
    const _send = XMLHttpRequest.prototype.send;
    XMLHttpRequest.prototype.open = function(m, u) {{
        this.__cm = m; this.__cu = String(u); this.__ch = {{}};
        return _open.apply(this, arguments);
    }};
    XMLHttpRequest.prototype.send = function(body) {{
        const xhr = this;
        const match = (window.__clawser_patterns || []).find(p => xhr.__cu?.includes(p));
        if (match) {{
            xhr.addEventListener('load', function() {{
                window.__clawser_captures.push({{
                    pattern: match, method: xhr.__cm, url: xhr.__cu,
                    request_headers: xhr.__ch,
                    request_body: typeof body === 'string' ? body : null,
                    status: xhr.status,
                    response_headers: null,
                    response_body: xhr.responseText
                }});
            }});
        }}
        return _send.apply(this, arguments);
    }};
}})();
"#, p = patterns_json);
                let _ = page.evaluate_on_new_document(hook).await;
            }
        }

        Ok(Browser {
            inner: browser,
            _handler: handler_handle,
            ws_events,
        })
    }
}

// --- Browser ---

impl Browser {
    pub fn builder() -> BrowserBuilder {
        BrowserBuilder {
            headless: true,
            config_path: None,
            profile_index: None,
            seed_index: None,
            watch_patterns: Vec::new(),
            watch_ws_patterns: Vec::new(),
        }
    }

    pub async fn navigate(&self, url: &str) -> io::Result<Page> {
        let page = self.inner.new_page(url).await
            .map_err(|e| io::Error::other(format!("navigate failed: {}", e)))?;

        // Set up WS event listener on this page
        let ws_events = self.ws_events.clone();
        let mut ws_listener = page.event_listener::<EventWebSocketCreated>().await
            .map_err(|e| io::Error::other(format!("ws listener failed: {}", e)))?;
        let events_clone = ws_events.clone();
        tokio::spawn(async move {
            while let Some(ev) = ws_listener.next().await {
                let val = serde_json::json!({
                    "type": "created",
                    "url": ev.url,
                    "request_id": ev.request_id.inner().to_string(),
                });
                events_clone.lock().await.push(val);
            }
        });

        // Also listen for WS frames
        let mut frame_listener = page.event_listener::<EventWebSocketFrameReceived>().await
            .map_err(|e| io::Error::other(format!("ws frame listener failed: {}", e)))?;
        let events_clone2 = ws_events.clone();
        tokio::spawn(async move {
            while let Some(ev) = frame_listener.next().await {
                let val = serde_json::json!({
                    "type": "frame_received",
                    "request_id": ev.request_id.inner().to_string(),
                    "data": ev.response.payload_data,
                });
                events_clone2.lock().await.push(val);
            }
        });

        Ok(Page {
            inner: page,
            ws_events,
        })
    }

    pub async fn cookies(&self, url: &str) -> io::Result<Vec<Cookie>> {
        let pages = self.inner.pages().await
            .map_err(|e| io::Error::other(format!("pages failed: {}", e)))?;
        if let Some(page) = pages.first() {
            let cdp_cookies = page.get_cookies().await
                .map_err(|e| io::Error::other(format!("cookies failed: {}", e)))?;
            let cookies: Vec<Cookie> = cdp_cookies.into_iter()
                .filter(|c| url.is_empty() || c.domain.contains(url.split('/').nth(2).unwrap_or("")))
                .map(|c| Cookie {
                    name: c.name,
                    value: c.value,
                    domain: c.domain,
                    path: c.path,
                    secure: c.secure,
                    httponly: c.http_only,
                })
                .collect();
            Ok(cookies)
        } else {
            Ok(vec![])
        }
    }

    pub async fn screenshot(&self) -> io::Result<Vec<u8>> {
        let pages = self.inner.pages().await
            .map_err(|e| io::Error::other(format!("pages failed: {}", e)))?;
        if let Some(page) = pages.first() {
            page.screenshot(
                chromiumoxide::page::ScreenshotParams::builder().build()
            ).await
                .map_err(|e| io::Error::other(format!("screenshot failed: {}", e)))
        } else {
            Err(io::Error::other("no page"))
        }
    }

    pub async fn shutdown(mut self) -> io::Result<()> {
        self.inner.close().await
            .map_err(|e| io::Error::other(format!("close failed: {}", e)))?;
        self._handler.abort();
        Ok(())
    }
}

// --- Page ---

impl Page {
    pub async fn js(&self, code: &str) -> io::Result<String> {
        let result = self.inner.evaluate(code).await
            .map_err(|e| io::Error::other(format!("js failed: {}", e)))?;
        match result.value() {
            Some(val) => match val {
                serde_json::Value::String(s) => Ok(s.clone()),
                other => Ok(other.to_string()),
            },
            None => Ok(String::new()),
        }
    }

    pub async fn mouse_move(&self, x: f64, y: f64, steps: u32) -> io::Result<()> {
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
            let px = u*u*u*start_x + 3.0*u*u*t*cp1x + 3.0*u*t*t*cp2x + t*t*t*x;
            let py = u*u*u*start_y + 3.0*u*u*t*cp1y + 3.0*u*t*t*cp2y + t*t*t*y;
            self.inner.move_mouse(chromiumoxide::layout::Point::new(px, py)).await
                .map_err(|e| io::Error::other(format!("mouse_move failed: {}", e)))?;
            let base_ms = 5 + ((r0.wrapping_add(i as u64 * 7)) % 12);
            tokio::time::sleep(Duration::from_millis(base_ms)).await;
        }
        Ok(())
    }

    pub async fn click(&self, x: f64, y: f64) -> io::Result<()> {
        self.mouse_move(x, y, 15).await?;
        self.inner.click(chromiumoxide::layout::Point::new(x, y)).await
            .map_err(|e| io::Error::other(format!("click failed: {}", e)))?;
        Ok(())
    }

    pub async fn type_text(&self, text: &str) -> io::Result<()> {
        use chromiumoxide::cdp::browser_protocol::input::{
            DispatchKeyEventParams, DispatchKeyEventType,
        };
        for ch in text.chars() {
            let key_str = ch.to_string();
            let _ = self.inner.execute(
                DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::KeyDown)
                    .text(&key_str)
                    .key(&key_str)
                    .build()
                    .unwrap()
            ).await;
            let mut buf = [0u8; 8];
            profiles::getrandom(&mut buf);
            let dwell = 30 + (u64::from_le_bytes(buf) % 50);
            tokio::time::sleep(Duration::from_millis(dwell)).await;
            let _ = self.inner.execute(
                DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::KeyUp)
                    .key(&key_str)
                    .build()
                    .unwrap()
            ).await;
            profiles::getrandom(&mut buf);
            let gap = 40 + (u64::from_le_bytes(buf) % 140);
            tokio::time::sleep(Duration::from_millis(gap)).await;
        }
        Ok(())
    }

    pub async fn scroll(&self, delta_y: i32) -> io::Result<()> {
        self.js(&format!(
            "window.scrollBy({{top: {}, behavior: 'smooth'}})", delta_y
        )).await?;
        tokio::time::sleep(Duration::from_millis(300)).await;
        Ok(())
    }

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

    pub async fn capture_mhtml(&self) -> io::Result<Vec<u8>> {
        use chromiumoxide::cdp::browser_protocol::page::CaptureSnapshotFormat;
        let data = self.inner.execute(
            chromiumoxide::cdp::browser_protocol::page::CaptureSnapshotParams::builder()
                .format(CaptureSnapshotFormat::Mhtml)
                .build()
        ).await
            .map_err(|e| io::Error::other(format!("mhtml failed: {}", e)))?;
        Ok(data.result.data.into_bytes())
    }

    pub async fn capture_html(&self) -> io::Result<String> {
        self.inner.content().await
            .map_err(|e| io::Error::other(format!("html failed: {}", e)))
    }

    /// Get all HTTP captures (from JS hooks).
    pub async fn captures(&self) -> io::Result<Vec<CapturedRequest>> {
        let result = self.js(
            "(function(){return JSON.stringify(window.__clawser_captures || [])})()"
        ).await?;
        if result.is_empty() || result == "[]" { return Ok(vec![]); }
        serde_json::from_str(&result)
            .map_err(|e| io::Error::other(format!("parse captures failed: {}", e)))
    }

    /// Wait for HTTP capture matching pattern.
    pub async fn wait_for_capture(&self, pattern: &str, timeout_ms: u64) -> io::Result<CapturedRequest> {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed().as_millis() as u64 > timeout_ms {
                return Err(io::Error::new(io::ErrorKind::TimedOut,
                    format!("No capture matching '{}' after {}ms", pattern, timeout_ms)));
            }
            let caps = self.captures().await?;
            if let Some(cap) = caps.into_iter().find(|c| c.url.contains(pattern)) {
                return Ok(cap);
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    /// Replay a captured HTTP request.
    pub async fn replay(&self, captured: &CapturedRequest) -> io::Result<String> {
        let headers_js = match &captured.request_headers {
            Some(h) => serde_json::to_string(h).unwrap_or("{}".into()),
            None => "{}".into(),
        };
        let body_js = match &captured.request_body {
            Some(b) => format!("body: {},", serde_json::to_string(b).unwrap_or("null".into())),
            None => String::new(),
        };
        let js = format!(
            r#"(async function(){{
                const resp = await fetch('{}', {{method:'{}',headers:{},{}}});
                return await resp.text();
            }})()"#,
            captured.url, captured.method, headers_js, body_js
        );
        self.js(&js).await
    }

    /// Get all detected WebSocket connections (CDP-level, works for workers + iframes).
    pub async fn detected_ws(&self) -> Vec<serde_json::Value> {
        self.ws_events.lock().await.clone()
    }

    /// Wait for a WebSocket matching URL pattern.
    pub async fn wait_for_ws(&self, pattern: &str, timeout_ms: u64) -> io::Result<String> {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed().as_millis() as u64 > timeout_ms {
                return Err(io::Error::new(io::ErrorKind::TimedOut,
                    format!("No WS matching '{}' after {}ms", pattern, timeout_ms)));
            }
            let events = self.ws_events.lock().await;
            for ev in events.iter() {
                if ev["type"] == "created" {
                    let url = ev["url"].as_str().unwrap_or("");
                    if url.contains(pattern) {
                        return Ok(url.to_string());
                    }
                }
            }
            drop(events);
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    /// Get received WS messages for a connection matching pattern.
    pub async fn ws_messages(&self, pattern: &str) -> Vec<String> {
        let events = self.ws_events.lock().await;
        // Find the request_id for matching WS
        let mut target_id = String::new();
        for ev in events.iter() {
            if ev["type"] == "created" && ev["url"].as_str().unwrap_or("").contains(pattern) {
                target_id = ev["request_id"].as_str().unwrap_or("").to_string();
                break;
            }
        }
        if target_id.is_empty() { return vec![]; }
        events.iter()
            .filter(|ev| ev["type"] == "frame_received" && ev["request_id"] == target_id)
            .filter_map(|ev| ev["data"].as_str().map(String::from))
            .collect()
    }
}

fn find_chrome_exe() -> io::Result<std::path::PathBuf> {
    if let Ok(path) = std::env::var("CLAWSER_CHROME_PATH") {
        let p = std::path::PathBuf::from(&path);
        if p.exists() { return Ok(p); }
    }
    if let Ok(exe) = std::env::current_exe() {
        let dir = exe.parent().unwrap_or(exe.as_ref());
        let candidate = dir.join(if cfg!(windows) { "chrome.exe" } else { "chrome" });
        if candidate.exists() { return Ok(candidate); }
    }
    let candidate = std::path::PathBuf::from("out/Default").join(if cfg!(windows) { "chrome.exe" } else { "chrome" });
    if candidate.exists() { return Ok(candidate); }

    Err(io::Error::new(io::ErrorKind::NotFound, "chrome.exe not found. Set CLAWSER_CHROME_PATH"))
}
