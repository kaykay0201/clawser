//! Antidetect browser automation powered by chromiumoxide CDP.
//!
//! ```rust,no_run
//! use clawser_browser::Browser;
//!
//! #[tokio::main]
//! async fn main() -> clawser_browser::Result<()> {
//!     let browser = Browser::builder()
//!         .headful()
//!         .profile(7, 777)
//!         .build().await?;
//!
//!     let page = browser.new_page("https://example.com").await?;
//!     let title = page.js("document.title").await?;
//!     println!("{title}");
//!     browser.close().await
//! }
//! ```

mod client;
mod profile;

pub use client::{HttpClient, HttpClientBuilder};
pub use profile::{generate_config_json, random_profile, write_config_file, HwProfile, PROFILES};

// Re-export wreq types that users need
pub use wreq;
pub use wreq::header;

use chromiumoxide::browser::{Browser as CdpBrowser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::input::{
    DispatchKeyEventParams, DispatchKeyEventType, DispatchMouseEventParams,
    DispatchMouseEventType, MouseButton,
};
use chromiumoxide::cdp::browser_protocol::network::Cookie;
use chromiumoxide::cdp::browser_protocol::page::{CaptureScreenshotParams, NavigateParams};
use chromiumoxide::Page as CdpPage;
use futures_util::StreamExt;
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinHandle;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ── BrowserBuilder ──────────────────────────────────────────────

pub struct BrowserBuilder {
    headless: bool,
    chrome_path: Option<String>,
    profile_choice: ProfileChoice,
    user_data_dir: Option<String>,
    window_size: (u32, u32),
    proxy: Option<String>,
    extra_args: Vec<String>,
}

enum ProfileChoice {
    None,
    Random,
    Indexed { index: usize, seed: u64 },
    ConfigFile(String),
}

impl Default for BrowserBuilder {
    fn default() -> Self {
        Self {
            headless: true,
            chrome_path: None,
            profile_choice: ProfileChoice::None,
            user_data_dir: None,
            window_size: (1920, 1080),
            proxy: None,
            extra_args: Vec::new(),
        }
    }
}

impl BrowserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run with visible window.
    pub fn headful(mut self) -> Self {
        self.headless = false;
        self
    }

    /// Run in headless mode (default).
    pub fn headless(mut self) -> Self {
        self.headless = true;
        self
    }

    /// Path to chrome.exe. Falls back to `CLAWSER_CHROME_PATH` env.
    pub fn chrome_path(mut self, path: impl Into<String>) -> Self {
        self.chrome_path = Some(path.into());
        self
    }

    /// Use hardware profile `index` (0..99) with deterministic `seed`.
    pub fn profile(mut self, index: usize, seed: u64) -> Self {
        self.profile_choice = ProfileChoice::Indexed { index, seed };
        self
    }

    /// Use a random hardware profile.
    pub fn random(mut self) -> Self {
        self.profile_choice = ProfileChoice::Random;
        self
    }

    /// Use a custom clawser config JSON file path.
    pub fn config(mut self, path: impl Into<String>) -> Self {
        self.profile_choice = ProfileChoice::ConfigFile(path.into());
        self
    }

    /// Persistent user data dir for cookies/storage.
    pub fn user_data_dir(mut self, path: impl Into<String>) -> Self {
        self.user_data_dir = Some(path.into());
        self
    }

    /// Window size (default 1920x1080).
    pub fn window_size(mut self, w: u32, h: u32) -> Self {
        self.window_size = (w, h);
        self
    }

    /// Set SOCKS5 proxy. Format: `"socks5://user:pass@host:port"`.
    /// Also accepts `"socks5://host:port"` (no auth).
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Set SOCKS5 proxy from host, port, user, pass components.
    pub fn proxy_socks5(mut self, host: &str, port: u16, user: &str, pass: &str) -> Self {
        self.proxy = Some(format!("socks5://{}:{}@{}:{}", user, pass, host, port));
        self
    }

    /// Add a Chrome launch argument (e.g. `"disable-gpu"`).
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.extra_args.push(arg.into());
        self
    }

    /// Launch the browser.
    pub async fn build(self) -> Result<Browser> {
        let chrome_path = self
            .chrome_path
            .or_else(|| std::env::var("CLAWSER_CHROME_PATH").ok())
            .ok_or("set CLAWSER_CHROME_PATH env or call .chrome_path()")?;

        let (config_path, profile_id) = match self.profile_choice {
            ProfileChoice::Random => {
                let (idx, seed) = profile::random_profile();
                let p = profile::write_config_file(idx, seed)?;
                (Some(p), Some(format!("clawser_{idx}_{seed}")))
            }
            ProfileChoice::Indexed { index, seed } => {
                let p = profile::write_config_file(index, seed)?;
                (Some(p), Some(format!("clawser_{index}_{seed}")))
            }
            ProfileChoice::ConfigFile(ref path) => (Some(PathBuf::from(path)), None),
            ProfileChoice::None => (None, None),
        };

        let mut cb = BrowserConfig::builder()
            .chrome_executable(&chrome_path)
            .disable_default_args()
            .no_sandbox()
            .with_head()
            .window_size(self.window_size.0, self.window_size.1)
            .viewport(None);

        if self.headless {
            cb = cb.arg(("headless", "new"));
        }

        if let Some(ref cp) = config_path {
            let p = cp.to_string_lossy().replace('/', "\\");
            cb = cb.arg(("clawser-config", p.as_str()));
        }

        if let Some(ref udd) = self.user_data_dir {
            cb = cb.user_data_dir(udd);
        } else if let Some(ref id) = profile_id {
            // Store profiles next to the chrome exe (project dir), not in %TEMP%.
            let profiles_dir = std::path::Path::new(&chrome_path)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("clawser_profiles")
                .join(id);
            cb = cb.user_data_dir(profiles_dir);
        }

        if let Some(ref proxy) = self.proxy {
            cb = cb.arg(("proxy-server", proxy.as_str()));
        }

        cb = cb
            .arg(("disable-blink-features", "AutomationControlled"))
            .arg(("remote-allow-origins", "*"))
            .arg("no-first-run")
            .arg("no-default-browser-check");

        for a in &self.extra_args {
            cb = cb.arg(a.as_str());
        }

        let config = cb.build().map_err(|e| format!("browser config: {e}"))?;
        let (browser, mut handler) = CdpBrowser::launch(config).await?;

        let handle = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        Ok(Browser {
            inner: browser,
            _handler: handle,
            _config_path: config_path,
        })
    }
}

// ── Browser ─────────────────────────────────────────────────────

pub struct Browser {
    inner: CdpBrowser,
    _handler: JoinHandle<()>,
    _config_path: Option<PathBuf>,
}

impl Browser {
    pub fn builder() -> BrowserBuilder {
        BrowserBuilder::new()
    }

    /// Connect to an already-running Chrome at `ws_url`.
    pub async fn connect(ws_url: &str) -> Result<Self> {
        let (browser, mut handler) = CdpBrowser::connect(ws_url).await?;
        let handle = tokio::spawn(async move {
            while let Some(e) = handler.next().await {
                if e.is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            inner: browser,
            _handler: handle,
            _config_path: None,
        })
    }

    /// Open a new tab and navigate to URL.
    /// Background human simulation (mouse movement + scrolling) starts automatically.
    pub async fn new_page(&self, url: &str) -> Result<Page> {
        let page = self.inner.new_page(url).await?;
        let (tx, rx) = tokio::sync::watch::channel(false);
        let sim_page = page.clone();
        tokio::spawn(async move {
            human_loop(sim_page, rx, 1920.0, 1080.0).await;
        });
        Ok(Page { inner: page, _sim_cancel: tx })
    }

    /// Get all open pages (each gets its own background human simulation).
    pub async fn pages(&self) -> Result<Vec<Page>> {
        let mut result = Vec::new();
        for p in self.inner.pages().await? {
            let (tx, rx) = tokio::sync::watch::channel(false);
            let sim_page = p.clone();
            tokio::spawn(async move {
                human_loop(sim_page, rx, 1920.0, 1080.0).await;
            });
            result.push(Page { inner: p, _sim_cancel: tx });
        }
        Ok(result)
    }

    /// Get all browser cookies.
    pub async fn cookies(&self) -> Result<Vec<Cookie>> {
        Ok(self.inner.get_cookies().await?)
    }

    /// Access underlying chromiumoxide Browser for raw CDP.
    pub fn cdp(&self) -> &CdpBrowser {
        &self.inner
    }

    /// Gracefully shut down the browser.
    pub async fn close(mut self) -> Result<()> {
        self.inner.close().await?;
        let _ = self._handler.await;
        Ok(())
    }
}

// ── Page ────────────────────────────────────────────────────────

pub struct Page {
    inner: CdpPage,
    _sim_cancel: tokio::sync::watch::Sender<bool>,
}

impl Page {
    /// Navigate to URL (returns immediately, does not wait for load).
    pub async fn navigate(&self, url: &str) -> Result<()> {
        self.inner.execute(NavigateParams::new(url)).await?;
        Ok(())
    }

    /// Navigate to URL and wait for page load.
    pub async fn goto(&self, url: &str) -> Result<()> {
        self.inner.goto(url).await?;
        Ok(())
    }

    /// Wait for next navigation/load event.
    pub async fn wait_for_load(&self) -> Result<()> {
        self.inner.wait_for_navigation().await?;
        Ok(())
    }

    /// Run JS expression in page context, return result as String.
    pub async fn js(&self, expr: &str) -> Result<String> {
        let result = self.inner.evaluate(expr).await?;
        match result.value() {
            Some(serde_json::Value::String(s)) => Ok(s.clone()),
            Some(serde_json::Value::Null) | None => Ok(String::new()),
            Some(v) => Ok(v.to_string()),
        }
    }

    /// Run JS and deserialize result into `T`.
    pub async fn js_as<T: serde::de::DeserializeOwned>(&self, expr: &str) -> Result<T> {
        let result = self.inner.evaluate(expr).await?;
        Ok(result.into_value()?)
    }

    /// Inject script that runs on every new document before page JS.
    pub async fn js_on_new_document(&self, script: &str) -> Result<()> {
        self.inner.evaluate_on_new_document(script).await?;
        Ok(())
    }

    /// Current page URL.
    pub async fn url(&self) -> Result<String> {
        Ok(self.inner.url().await?.unwrap_or_default())
    }

    /// Page title.
    pub async fn title(&self) -> Result<String> {
        Ok(self.inner.get_title().await?.unwrap_or_default())
    }

    /// Full page HTML.
    pub async fn html(&self) -> Result<String> {
        Ok(self.inner.content().await?)
    }

    /// Capture PNG screenshot.
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        Ok(self
            .inner
            .screenshot(CaptureScreenshotParams::default())
            .await?)
    }

    /// Click at (x, y) with realistic mouse movement + timing.
    pub async fn click(&self, x: f64, y: f64) -> Result<()> {
        self.inner
            .execute(DispatchMouseEventParams::new(
                DispatchMouseEventType::MouseMoved,
                x,
                y,
            ))
            .await?;
        tokio::time::sleep(jitter(20, 60)).await;

        let mut press =
            DispatchMouseEventParams::new(DispatchMouseEventType::MousePressed, x, y);
        press.button = Some(MouseButton::Left);
        press.click_count = Some(1);
        self.inner.execute(press).await?;
        tokio::time::sleep(jitter(40, 120)).await;

        let mut release =
            DispatchMouseEventParams::new(DispatchMouseEventType::MouseReleased, x, y);
        release.button = Some(MouseButton::Left);
        release.click_count = Some(1);
        self.inner.execute(release).await?;
        Ok(())
    }

    /// Type text into focused element with realistic keystroke timing.
    pub async fn type_text(&self, text: &str) -> Result<()> {
        for ch in text.chars() {
            let s = ch.to_string();
            let mut down = DispatchKeyEventParams::new(DispatchKeyEventType::KeyDown);
            down.text = Some(s.clone());
            down.key = Some(s.clone());
            self.inner.execute(down).await?;

            let mut up = DispatchKeyEventParams::new(DispatchKeyEventType::KeyUp);
            up.key = Some(s);
            self.inner.execute(up).await?;

            tokio::time::sleep(jitter(30, 130)).await;
        }
        Ok(())
    }

    /// Scroll by `delta_y` pixels (positive = down).
    pub async fn scroll(&self, delta_y: f64) -> Result<()> {
        let mut ev =
            DispatchMouseEventParams::new(DispatchMouseEventType::MouseWheel, 400.0, 300.0);
        ev.delta_x = Some(0.0);
        ev.delta_y = Some(delta_y);
        self.inner.execute(ev).await?;
        Ok(())
    }

    /// Sleep for `ms` milliseconds.
    pub async fn wait(&self, ms: u64) {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }

    /// Close the page/tab.
    pub async fn close(self) -> Result<()> {
        self.inner.close().await?;
        Ok(())
    }

    /// Access underlying chromiumoxide Page for raw CDP.
    pub fn cdp(&self) -> &CdpPage {
        &self.inner
    }
}

// ── Human Simulation (built-in) ────────────────────────────────
//
// Every Page auto-starts a background task that generates realistic mouse
// movement + scrolling via CDP. The task lives as long as the Page — when
// the Page is dropped or the browser closes, the simulation stops.
// Safe: only moves the cursor and scrolls — never clicks or types.

/// Core simulation loop — runs until cancelled.
async fn human_loop(
    page: CdpPage,
    mut cancel: tokio::sync::watch::Receiver<bool>,
    vw: f64,
    vh: f64,
) {
    let mut rng = FastRng::new(nanos() ^ 0xDEAD_BEEF);
    let mut mouse_x: f64 = vw / 2.0;
    let mut mouse_y: f64 = vh / 2.0;

    loop {
        if *cancel.borrow() {
            break;
        }

        // Pick random action weighted: 60% mouse move, 25% scroll, 15% idle
        let roll = rng.next_range(100);
        if roll < 60 {
            // Mouse move along a bezier curve to a random target
            let tx = rng.next_f64() * (vw - 40.0) + 20.0;
            let ty = rng.next_f64() * (vh - 40.0) + 20.0;
            if bezier_move(&page, &mut cancel, &mut rng, mouse_x, mouse_y, tx, ty)
                .await
                .is_err()
            {
                break;
            }
            mouse_x = tx;
            mouse_y = ty;
        } else if roll < 85 {
            // Scroll: small random amount, sometimes up
            let direction = if rng.next_range(100) < 80 { 1.0 } else { -1.0 };
            let amount = (rng.next_range(200) as f64 + 50.0) * direction;
            let mut ev = DispatchMouseEventParams::new(
                DispatchMouseEventType::MouseWheel,
                mouse_x,
                mouse_y,
            );
            ev.delta_x = Some(0.0);
            ev.delta_y = Some(amount);
            let _ = page.execute(ev).await;
        } else {
            // Idle — just wait (simulates reading)
        }

        // Pause between actions: 800ms–4s
        let delay = rng.next_range(3200) as u64 + 800;
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(delay)) => {}
            _ = cancel.changed() => break,
        }
    }
}

/// Move mouse along a quadratic bezier curve from (sx,sy) to (tx,ty).
async fn bezier_move(
    page: &CdpPage,
    cancel: &mut tokio::sync::watch::Receiver<bool>,
    rng: &mut FastRng,
    sx: f64,
    sy: f64,
    tx: f64,
    ty: f64,
) -> std::result::Result<(), ()> {
    // Random control point (gives the curve a natural arc)
    let cx = (sx + tx) / 2.0 + (rng.next_f64() - 0.5) * 200.0;
    let cy = (sy + ty) / 2.0 + (rng.next_f64() - 0.5) * 200.0;

    let steps = rng.next_range(10) + 8; // 8–17 intermediate points
    for i in 1..=steps {
        if *cancel.borrow() {
            return Err(());
        }
        let t = i as f64 / steps as f64;
        let inv = 1.0 - t;
        // Quadratic bezier: B(t) = (1-t)²·S + 2(1-t)t·C + t²·T
        let x = inv * inv * sx + 2.0 * inv * t * cx + t * t * tx;
        let y = inv * inv * sy + 2.0 * inv * t * cy + t * t * ty;

        let ev =
            DispatchMouseEventParams::new(DispatchMouseEventType::MouseMoved, x, y);
        let _ = page.execute(ev).await;

        // Inter-step delay: 8–25ms (realistic mouse polling rate)
        let step_delay = rng.next_range(17) as u64 + 8;
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(step_delay)) => {}
            _ = cancel.changed() => return Err(()),
        }
    }
    Ok(())
}

// ── Fast PRNG ──────────────────────────────────────────────────

struct FastRng {
    state: u64,
}

impl FastRng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x1234_5678_9ABC_DEF0 } else { seed },
        }
    }

    fn next(&mut self) -> u64 {
        let mut s = self.state;
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        self.state = s;
        s
    }

    fn next_range(&mut self, max: u32) -> u32 {
        (self.next() % max as u64) as u32
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() & 0x000F_FFFF_FFFF_FFFF) as f64 / (0x0010_0000_0000_0000u64 as f64)
    }
}

// ── Helpers ─────────────────────────────────────────────────────

fn nanos() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64
}

fn jitter(min_ms: u64, max_ms: u64) -> std::time::Duration {
    std::time::Duration::from_millis(min_ms + nanos() % (max_ms - min_ms))
}
