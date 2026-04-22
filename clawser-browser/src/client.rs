//! Lightweight HTTP client that impersonates Chrome 134 at the TLS/HTTP2 level.
//!
//! Uses wreq (BoringSSL) to produce identical JA3/JA4 and HTTP/2 fingerprints
//! as the real Chromium binary. ~50-100x faster than launching a browser.
//!
//! ```rust,no_run
//! use clawser_browser::HttpClient;
//!
//! #[tokio::main]
//! async fn main() -> clawser_browser::Result<()> {
//!     let client = HttpClient::new()?;
//!     let resp = client.get("https://example.com").send().await?;
//!     println!("{}", resp.text().await?);
//!     Ok(())
//! }
//! ```

use crate::profile::generate_config_json;
use crate::Result;
use wreq_util::{Emulation, EmulationOS, EmulationOption};

/// HTTP client impersonating Chrome 134 at TLS + HTTP/2 level.
///
/// Produces identical JA3/JA4, HTTP/2 SETTINGS, pseudo-header ordering,
/// and default headers as a real Chrome 134 browser on Windows.
pub struct HttpClient {
    inner: wreq::Client,
    ua: String,
}

impl HttpClient {
    /// Create a client impersonating Chrome 134 on Windows with default settings.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    /// Start building a custom client.
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::default()
    }

    // ── Request methods ────────────────────────────────────────

    /// Start a GET request.
    pub fn get(&self, url: &str) -> wreq::RequestBuilder {
        self.inner.get(url)
    }

    /// Start a POST request.
    pub fn post(&self, url: &str) -> wreq::RequestBuilder {
        self.inner.post(url)
    }

    /// Start a PUT request.
    pub fn put(&self, url: &str) -> wreq::RequestBuilder {
        self.inner.put(url)
    }

    /// Start a DELETE request.
    pub fn delete(&self, url: &str) -> wreq::RequestBuilder {
        self.inner.delete(url)
    }

    /// Start a PATCH request.
    pub fn patch(&self, url: &str) -> wreq::RequestBuilder {
        self.inner.patch(url)
    }

    /// Start a HEAD request.
    pub fn head(&self, url: &str) -> wreq::RequestBuilder {
        self.inner.head(url)
    }

    /// Build a request with any HTTP method.
    pub fn request(&self, method: wreq::Method, url: &str) -> wreq::RequestBuilder {
        self.inner.request(method, url)
    }

    // ── Accessors ──────────────────────────────────────────────

    /// User-Agent string this client sends.
    pub fn user_agent(&self) -> &str {
        &self.ua
    }

    /// Access the underlying wreq::Client for advanced usage.
    pub fn wreq(&self) -> &wreq::Client {
        &self.inner
    }
}

// ── Builder ────────────────────────────────────────────────────

/// Builder for [`HttpClient`].
pub struct HttpClientBuilder {
    profile: Option<(usize, u64)>,
    proxy: Option<String>,
    timeout_secs: u64,
    cookie_store: bool,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self {
            profile: None,
            proxy: None,
            timeout_secs: 30,
            cookie_store: true,
        }
    }
}

impl HttpClientBuilder {
    /// Use a specific hardware profile (same index/seed as Browser::builder().profile()).
    /// Extracts matching User-Agent and Accept-Language from the profile config.
    pub fn profile(mut self, index: usize, seed: u64) -> Self {
        self.profile = Some((index, seed));
        self
    }

    /// Set an HTTP/HTTPS/SOCKS5 proxy.
    /// Examples: `"http://proxy:8080"`, `"socks5://user:pass@proxy:1080"`
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Request timeout in seconds (default: 30).
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Enable/disable cookie jar (default: true).
    pub fn cookie_store(mut self, enable: bool) -> Self {
        self.cookie_store = enable;
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<HttpClient> {
        // Extract UA and languages from profile config if provided
        let (ua, languages) = if let Some((index, seed)) = self.profile {
            let json = generate_config_json(index, seed);
            let config: serde_json::Value = serde_json::from_str(&json)?;
            let ua = config["navigator"]["user_agent"]
                .as_str()
                .unwrap_or(DEFAULT_UA)
                .to_string();
            let langs: Vec<String> = config["navigator"]["languages"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_else(|| vec!["en-US".into(), "en".into()]);
            (ua, langs.join(", "))
        } else {
            (DEFAULT_UA.to_string(), "en-US, en".to_string())
        };

        // Chrome 134 Windows emulation — matches our Chromium binary's TLS/HTTP2
        let emulation = EmulationOption::builder()
            .emulation(Emulation::Chrome134)
            .emulation_os(EmulationOS::Windows)
            .skip_headers(true) // we set our own headers
            .build();

        let mut cb = wreq::Client::builder()
            .emulation(emulation)
            .cookie_store(self.cookie_store)
            .user_agent(&ua)
            .default_headers(build_chrome_headers(&ua, &languages))
            .timeout(std::time::Duration::from_secs(self.timeout_secs));

        if let Some(ref proxy_url) = self.proxy {
            cb = cb.proxy(wreq::Proxy::all(proxy_url)?);
        }

        let client = cb.build()?;

        Ok(HttpClient { inner: client, ua })
    }
}

// ── Internals ──────────────────────────────────────────────────

const DEFAULT_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.6998.0 Safari/537.36";

const SEC_CH_UA: &str = r#""Chromium";v="134", "Not:A-Brand";v="24", "Google Chrome";v="134""#;

/// Build default headers matching a real Chrome 134 request.
fn build_chrome_headers(ua: &str, languages: &str) -> wreq::header::HeaderMap {
    use wreq::header::{self, HeaderMap, HeaderValue};

    let mut h = HeaderMap::new();

    // Order matters — Chrome sends these in a specific sequence
    h.insert(
        header::ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7"),
    );
    h.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );
    if let Ok(v) = HeaderValue::from_str(languages) {
        h.insert(header::ACCEPT_LANGUAGE, v);
    }
    h.insert("sec-ch-ua", HeaderValue::from_static(SEC_CH_UA));
    h.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    h.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));
    h.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    h.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    h.insert("sec-fetch-site", HeaderValue::from_static("none"));
    h.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    h.insert(
        header::UPGRADE_INSECURE_REQUESTS,
        HeaderValue::from_static("1"),
    );
    if let Ok(v) = HeaderValue::from_str(ua) {
        h.insert(header::USER_AGENT, v);
    }

    h
}
