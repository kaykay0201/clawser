//! # clawser-fetch
//!
//! Chromium-powered HTTP client with realistic browser fingerprints.
//!
//! Uses a real Chromium network stack (TLS, HTTP/2, cookies) with spoofed
//! browser identity to produce requests indistinguishable from a real Chrome
//! browser. Defeats WAF/bot-detection fingerprinting (Akamai, Cloudflare, etc).
//!
//! ## Quick start
//!
//! ```no_run
//! use clawser_fetch::{Session, Seed};
//!
//! // Random browser identity (seed returned for replay).
//! let (session, seed) = Session::random().unwrap();
//! println!("Save this seed: {:?}", seed);
//!
//! // Simple GET.
//! let resp = session.get("https://example.com").unwrap();
//! println!("{} {}", resp.status(), resp.url());
//! println!("{}", resp.text().unwrap());
//!
//! // Replay same identity later.
//! let session2 = Session::from_seed(&seed).unwrap();
//! ```
//!
//! ## Requirements
//!
//! Place `clawser_fetch.dll` (Windows) or `libclawser_fetch.so` (Linux) where
//! your binary can find it, or set `CLAWSER_LIB_PATH` to its full path.

mod ffi;

use std::ffi::{CStr, CString};
use std::fmt;

use serde::{Deserialize, Serialize};

// Re-export the seed type.
pub use ffi::ClawserSeed as Seed;

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

/// Error type for clawser operations.
#[derive(Debug, Clone)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

fn last_error() -> String {
    let lib = ffi::lib().unwrap();
    unsafe {
        let ptr = (lib.last_error)();
        if ptr.is_null() {
            return "unknown error".into();
        }
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

/// A browser session with a fixed fingerprint identity.
///
/// Owns cookies, default headers, and the Chromium network context.
/// All requests from the same session share cookies and identity.
pub struct Session {
    ptr: *mut ffi::ClawserSession,
}

// The C layer uses a dedicated IO thread per session — Send is safe.
unsafe impl Send for Session {}

impl Session {
    /// Create a session with a randomly generated browser identity.
    ///
    /// Returns the session and the seed used, so you can replay the same
    /// identity later with [`Session::from_seed`].
    pub fn random() -> Result<(Self, Seed)> {
        let lib = ffi::lib().map_err(Error)?;
        let mut seed = Seed {
            hw_seed: 0,
            canvas_seed: 0,
            webgl_seed: 0,
            audio_seed: 0,
            client_rects_seed: 0,
        };
        let ptr = unsafe { (lib.session_create_random)(&mut seed) };
        if ptr.is_null() {
            return Err(Error(last_error()));
        }
        Ok((Self { ptr }, seed))
    }

    /// Recreate a session from a previously saved seed (deterministic).
    pub fn from_seed(seed: &Seed) -> Result<Self> {
        let lib = ffi::lib().map_err(Error)?;
        let ptr = unsafe { (lib.session_from_seed)(seed) };
        if ptr.is_null() {
            return Err(Error(last_error()));
        }
        Ok(Self { ptr })
    }

    /// Create a session from a JSON config file (advanced, full control).
    pub fn from_config(path: &str) -> Result<Self> {
        let lib = ffi::lib().map_err(Error)?;
        let c_path = CString::new(path).map_err(|e| Error(e.to_string()))?;
        let ptr = unsafe { (lib.session_create)(c_path.as_ptr()) };
        if ptr.is_null() {
            return Err(Error(last_error()));
        }
        Ok(Self { ptr })
    }

    /// Get all cookies as a JSON string.
    pub fn cookies_json(&self) -> String {
        let lib = ffi::lib().unwrap();
        unsafe {
            let ptr = (lib.session_get_cookies)(self.ptr);
            if ptr.is_null() {
                return "[]".into();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    /// Get all cookies as parsed structs.
    pub fn cookies(&self) -> Vec<Cookie> {
        let json = self.cookies_json();
        serde_json::from_str(&json).unwrap_or_default()
    }

    /// Import cookies from a Vec<Cookie> (e.g. from clawser_browser).
    /// Returns number of cookies imported.
    pub fn import_cookies(&self, cookies: &[Cookie]) -> Result<usize> {
        let json = serde_json::to_string(cookies)
            .map_err(|e| Error(format!("serialize cookies: {}", e)))?;
        let lib = ffi::lib().map_err(Error)?;
        let c_json = CString::new(json).map_err(|e| Error(e.to_string()))?;
        let count = unsafe { (lib.session_import_cookies)(self.ptr, c_json.as_ptr()) };
        if count < 0 {
            return Err(Error(last_error()));
        }
        Ok(count as usize)
    }

    /// Set a single cookie for a URL.
    pub fn set_cookie(&self, url: &str, cookie: &Cookie) -> Result<()> {
        let lib = ffi::lib().map_err(Error)?;
        let json = serde_json::to_string(cookie)
            .map_err(|e| Error(format!("serialize cookie: {}", e)))?;
        let c_url = CString::new(url).map_err(|e| Error(e.to_string()))?;
        let c_json = CString::new(json).map_err(|e| Error(e.to_string()))?;
        let rc = unsafe { (lib.session_set_cookie)(self.ptr, c_url.as_ptr(), c_json.as_ptr()) };
        if rc < 0 {
            return Err(Error(last_error()));
        }
        Ok(())
    }

    /// Clear all cookies.
    pub fn clear_cookies(&self) {
        if let Ok(lib) = ffi::lib() {
            unsafe { (lib.session_clear_cookies)(self.ptr) };
        }
    }

    /// Start building a GET request.
    pub fn get(&self, url: &str) -> Result<Response> {
        self.request("GET", url).send()
    }

    /// Start building a request with the given method and URL.
    pub fn request(&self, method: &str, url: &str) -> RequestBuilder<'_> {
        RequestBuilder::new(self, method, url)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Ok(lib) = ffi::lib() {
            unsafe { (lib.session_destroy)(self.ptr) };
        }
    }
}

/// Builder for an HTTP request.
pub struct RequestBuilder<'a> {
    session: &'a Session,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    max_redirects: Option<i32>,
    timeout_ms: Option<u32>,
}

impl<'a> RequestBuilder<'a> {
    fn new(session: &'a Session, method: &str, url: &str) -> Self {
        Self {
            session,
            method: method.into(),
            url: url.into(),
            headers: Vec::new(),
            body: None,
            max_redirects: None,
            timeout_ms: None,
        }
    }

    /// Add a header to the request.
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Set the request body.
    pub fn body(mut self, data: &[u8]) -> Self {
        self.body = Some(data.to_vec());
        self
    }

    /// Set the request body as a string.
    pub fn body_str(mut self, data: &str) -> Self {
        self.body = Some(data.as_bytes().to_vec());
        self
    }

    /// Set maximum number of redirects to follow. 0 = don't follow.
    pub fn max_redirects(mut self, n: i32) -> Self {
        self.max_redirects = Some(n);
        self
    }

    /// Set request timeout in milliseconds.
    pub fn timeout_ms(mut self, ms: u32) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Send the request and block until the response is received.
    pub fn send(self) -> Result<Response> {
        let lib = ffi::lib().map_err(Error)?;
        let c_method = CString::new(self.method).map_err(|e| Error(e.to_string()))?;
        let c_url = CString::new(self.url).map_err(|e| Error(e.to_string()))?;

        let req = unsafe {
            (lib.request_new)(self.session.ptr, c_method.as_ptr(), c_url.as_ptr())
        };
        if req.is_null() {
            return Err(Error(last_error()));
        }

        for (name, value) in &self.headers {
            let c_name = CString::new(name.as_str()).map_err(|e| Error(e.to_string()))?;
            let c_value = CString::new(value.as_str()).map_err(|e| Error(e.to_string()))?;
            unsafe { (lib.request_set_header)(req, c_name.as_ptr(), c_value.as_ptr()) };
        }

        if let Some(body) = &self.body {
            unsafe { (lib.request_set_body)(req, body.as_ptr(), body.len()) };
        }

        if let Some(n) = self.max_redirects {
            unsafe { (lib.request_set_max_redirects)(req, n) };
        }

        if let Some(ms) = self.timeout_ms {
            unsafe { (lib.request_set_timeout_ms)(req, ms) };
        }

        // request_send consumes the request pointer.
        let resp = unsafe { (lib.request_send)(req) };
        if resp.is_null() {
            return Err(Error(last_error()));
        }

        Ok(Response { ptr: resp })
    }
}

/// An HTTP response.
pub struct Response {
    ptr: *mut ffi::ClawserResponse,
}

unsafe impl Send for Response {}

impl Response {
    /// HTTP status code (e.g., 200, 403).
    pub fn status(&self) -> i32 {
        let lib = ffi::lib().unwrap();
        unsafe { (lib.response_status_code)(self.ptr) }
    }

    /// Get a response header by name. Returns `None` if not present.
    pub fn header(&self, name: &str) -> Option<String> {
        let lib = ffi::lib().unwrap();
        let c_name = CString::new(name).ok()?;
        unsafe {
            let ptr = (lib.response_header)(self.ptr, c_name.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
            }
        }
    }

    /// Get all response headers as (name, value) pairs.
    pub fn headers(&self) -> Vec<(String, String)> {
        let lib = ffi::lib().unwrap();
        let count = unsafe { (lib.response_header_count)(self.ptr) };
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            unsafe {
                let name = (lib.response_header_name_at)(self.ptr, i);
                let value = (lib.response_header_value_at)(self.ptr, i);
                if !name.is_null() && !value.is_null() {
                    result.push((
                        CStr::from_ptr(name).to_string_lossy().into_owned(),
                        CStr::from_ptr(value).to_string_lossy().into_owned(),
                    ));
                }
            }
        }
        result
    }

    /// Response body as raw bytes.
    pub fn bytes(&self) -> &[u8] {
        let lib = ffi::lib().unwrap();
        unsafe {
            let ptr = (lib.response_body)(self.ptr);
            let len = (lib.response_body_len)(self.ptr);
            if ptr.is_null() || len == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(ptr, len)
            }
        }
    }

    /// Response body as a UTF-8 string.
    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.bytes().to_vec())
            .map_err(|e| Error(format!("response body is not valid UTF-8: {}", e)))
    }

    /// Final URL after redirects.
    pub fn url(&self) -> String {
        let lib = ffi::lib().unwrap();
        unsafe {
            let ptr = (lib.response_url)(self.ptr);
            if ptr.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        if let Ok(lib) = ffi::lib() {
            unsafe { (lib.response_destroy)(self.ptr) };
        }
    }
}

/// Returns the version string of the loaded clawser_fetch library.
pub fn version() -> Result<String> {
    let lib = ffi::lib().map_err(Error)?;
    unsafe {
        let ptr = (lib.version)();
        if ptr.is_null() {
            return Ok("unknown".into());
        }
        Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_roundtrip() {
        // Verify the Seed struct layout matches C.
        assert_eq!(std::mem::size_of::<Seed>(), 40); // 5 * u64
    }
}
