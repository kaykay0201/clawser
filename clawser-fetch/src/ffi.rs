use libloading::{Library, Symbol};
use std::ffi::{c_char, c_int, c_void};
use std::sync::OnceLock;

// Opaque C types.
pub type ClawserSession = c_void;
pub type ClawserRequest = c_void;
pub type ClawserResponse = c_void;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClawserSeed {
    pub hw_seed: u64,
    pub canvas_seed: u64,
    pub webgl_seed: u64,
    pub audio_seed: u64,
    pub client_rects_seed: u64,
}

/// All C API function signatures.
#[allow(dead_code)]
pub struct ClawserLib {
    _lib: Library,

    // Session lifecycle.
    pub session_create_random:
        unsafe extern "C" fn(*mut ClawserSeed) -> *mut ClawserSession,
    pub session_from_seed:
        unsafe extern "C" fn(*const ClawserSeed) -> *mut ClawserSession,
    pub session_create:
        unsafe extern "C" fn(*const c_char) -> *mut ClawserSession,
    pub session_get_cookies:
        unsafe extern "C" fn(*mut ClawserSession) -> *const c_char,
    pub session_set_cookie:
        unsafe extern "C" fn(*mut ClawserSession, *const c_char, *const c_char) -> c_int,
    pub session_import_cookies:
        unsafe extern "C" fn(*mut ClawserSession, *const c_char) -> c_int,
    pub session_clear_cookies:
        unsafe extern "C" fn(*mut ClawserSession),
    pub session_destroy: unsafe extern "C" fn(*mut ClawserSession),

    // Error.
    pub last_error: unsafe extern "C" fn() -> *const c_char,

    // Request building.
    pub request_new: unsafe extern "C" fn(
        *mut ClawserSession,
        *const c_char,
        *const c_char,
    ) -> *mut ClawserRequest,
    pub request_set_header:
        unsafe extern "C" fn(*mut ClawserRequest, *const c_char, *const c_char),
    pub request_set_body:
        unsafe extern "C" fn(*mut ClawserRequest, *const u8, usize),
    pub request_set_max_redirects:
        unsafe extern "C" fn(*mut ClawserRequest, c_int),
    pub request_set_timeout_ms:
        unsafe extern "C" fn(*mut ClawserRequest, u32),
    pub request_send:
        unsafe extern "C" fn(*mut ClawserRequest) -> *mut ClawserResponse,
    pub request_destroy: unsafe extern "C" fn(*mut ClawserRequest),

    // Response reading.
    pub response_status_code:
        unsafe extern "C" fn(*const ClawserResponse) -> c_int,
    pub response_header:
        unsafe extern "C" fn(*const ClawserResponse, *const c_char) -> *const c_char,
    pub response_header_count:
        unsafe extern "C" fn(*const ClawserResponse) -> usize,
    pub response_header_name_at:
        unsafe extern "C" fn(*const ClawserResponse, usize) -> *const c_char,
    pub response_header_value_at:
        unsafe extern "C" fn(*const ClawserResponse, usize) -> *const c_char,
    pub response_body:
        unsafe extern "C" fn(*const ClawserResponse) -> *const u8,
    pub response_body_len:
        unsafe extern "C" fn(*const ClawserResponse) -> usize,
    pub response_url:
        unsafe extern "C" fn(*const ClawserResponse) -> *const c_char,
    pub response_destroy: unsafe extern "C" fn(*mut ClawserResponse),

    // Version.
    pub version: unsafe extern "C" fn() -> *const c_char,
}

// Helper to load a symbol and transmute to fn pointer.
macro_rules! load_fn {
    ($lib:expr, $name:expr) => {{
        let sym: Symbol<*const c_void> = $lib
            .get($name)
            .map_err(|e| format!("failed to load {}: {}", String::from_utf8_lossy($name), e))?;
        std::mem::transmute(*sym)
    }};
}

impl ClawserLib {
    /// Load the clawser_fetch library from the given path.
    unsafe fn load_from(path: &str) -> Result<Self, String> {
        let lib = Library::new(path)
            .map_err(|e| format!("failed to load {}: {}", path, e))?;

        let c = ClawserLib {
            session_create_random: load_fn!(lib, b"clawser_session_create_random"),
            session_from_seed: load_fn!(lib, b"clawser_session_from_seed"),
            session_create: load_fn!(lib, b"clawser_session_create"),
            session_get_cookies: load_fn!(lib, b"clawser_session_get_cookies"),
            session_set_cookie: load_fn!(lib, b"clawser_session_set_cookie"),
            session_import_cookies: load_fn!(lib, b"clawser_session_import_cookies"),
            session_clear_cookies: load_fn!(lib, b"clawser_session_clear_cookies"),
            session_destroy: load_fn!(lib, b"clawser_session_destroy"),
            last_error: load_fn!(lib, b"clawser_last_error"),
            request_new: load_fn!(lib, b"clawser_request_new"),
            request_set_header: load_fn!(lib, b"clawser_request_set_header"),
            request_set_body: load_fn!(lib, b"clawser_request_set_body"),
            request_set_max_redirects: load_fn!(lib, b"clawser_request_set_max_redirects"),
            request_set_timeout_ms: load_fn!(lib, b"clawser_request_set_timeout_ms"),
            request_send: load_fn!(lib, b"clawser_request_send"),
            request_destroy: load_fn!(lib, b"clawser_request_destroy"),
            response_status_code: load_fn!(lib, b"clawser_response_status_code"),
            response_header: load_fn!(lib, b"clawser_response_header"),
            response_header_count: load_fn!(lib, b"clawser_response_header_count"),
            response_header_name_at: load_fn!(lib, b"clawser_response_header_name_at"),
            response_header_value_at: load_fn!(lib, b"clawser_response_header_value_at"),
            response_body: load_fn!(lib, b"clawser_response_body"),
            response_body_len: load_fn!(lib, b"clawser_response_body_len"),
            response_url: load_fn!(lib, b"clawser_response_url"),
            response_destroy: load_fn!(lib, b"clawser_response_destroy"),
            version: load_fn!(lib, b"clawser_version"),
            _lib: lib,
        };
        Ok(c)
    }
}

static LIB: OnceLock<Result<ClawserLib, String>> = OnceLock::new();

/// Initialize the library. Called automatically on first use, but you can call
/// it early to surface load errors.
///
/// Search order:
/// 1. `CLAWSER_LIB_PATH` env var (exact path to DLL/so)
/// 2. `clawser_fetch.dll` / `libclawser_fetch.so` in default search paths
pub fn lib() -> Result<&'static ClawserLib, String> {
    LIB.get_or_init(|| {
        let path = std::env::var("CLAWSER_LIB_PATH").unwrap_or_else(|_| {
            if cfg!(windows) {
                "clawser_fetch.dll".to_string()
            } else {
                "libclawser_fetch.so".to_string()
            }
        });
        unsafe { ClawserLib::load_from(&path) }
    })
    .as_ref()
    .map_err(|e| e.clone())
}
