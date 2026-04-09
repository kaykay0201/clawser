use std::env;
use std::io;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use tokio::process::{Child, Command};

/// GitHub release URL for auto-download.
const RELEASE_REPO: &str = "kaykay0201/just-fetch";
const RELEASE_VERSION: &str = "v0.1.0";

/// Find chrome.exe — searches in order:
/// 1. CLAWSER_CHROME_PATH env var
/// 2. Next to current exe
/// 3. ~/.clawser/chrome/ (auto-downloaded)
/// 4. out/Default/chrome.exe (dev)
/// If not found anywhere, auto-downloads from GitHub Release.
pub(crate) async fn find_chrome_exe() -> io::Result<PathBuf> {
    // 1. Env var
    if let Ok(path) = env::var("CLAWSER_CHROME_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 2. Next to current exe
    if let Ok(exe) = env::current_exe() {
        let dir = exe.parent().unwrap_or(exe.as_ref());
        let candidate = dir.join(chrome_exe_name());
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 3. Cached download location
    let cache_dir = chrome_cache_dir();
    let cached = cache_dir.join(chrome_exe_name());
    if cached.exists() {
        return Ok(cached);
    }

    // 4. Dev path
    let candidate = PathBuf::from("out/Default").join(chrome_exe_name());
    if candidate.exists() {
        return Ok(candidate);
    }

    // 5. Auto-download
    eprintln!("[clawser-browser] Chrome not found locally. Downloading from GitHub Release...");
    download_chrome(&cache_dir).await?;
    if cached.exists() {
        return Ok(cached);
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "Cannot find {}. Set CLAWSER_CHROME_PATH env var or ensure GitHub Release has assets.",
            chrome_exe_name()
        ),
    ))
}

/// ~/.clawser/chrome/
fn chrome_cache_dir() -> PathBuf {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".clawser").join("chrome")
}

/// Download chrome zip from GitHub Release and extract to cache_dir.
async fn download_chrome(cache_dir: &PathBuf) -> io::Result<()> {
    let asset_name = chrome_asset_name();
    let url = format!(
        "https://github.com/{}/releases/download/{}/{}",
        RELEASE_REPO, RELEASE_VERSION, asset_name
    );

    eprintln!("[clawser-browser] Downloading: {}", url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| io::Error::other(format!("HTTP client error: {}", e)))?;

    let resp = client
        .get(&url)
        .header("User-Agent", "clawser-browser")
        .send()
        .await
        .map_err(|e| io::Error::other(format!("Download failed: {}", e)))?;

    if !resp.status().is_success() {
        return Err(io::Error::other(format!(
            "Download failed: HTTP {} — ensure release {} has asset '{}'",
            resp.status(),
            RELEASE_VERSION,
            asset_name
        )));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| io::Error::other(format!("Download read failed: {}", e)))?;

    eprintln!(
        "[clawser-browser] Downloaded {} bytes. Extracting...",
        bytes.len()
    );

    // Create cache dir
    std::fs::create_dir_all(cache_dir)?;

    // Extract zip
    let cursor = std::io::Cursor::new(&bytes[..]);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| io::Error::other(format!("Zip open failed: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| io::Error::other(format!("Zip entry failed: {}", e)))?;
        let name = file.name().to_string();

        let out_path = cache_dir.join(&name);
        if name.ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
        }
    }

    eprintln!("[clawser-browser] Extracted to {}", cache_dir.display());
    Ok(())
}

fn chrome_asset_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "clawser-chrome-windows-x64.zip"
    } else if cfg!(target_os = "linux") {
        "clawser-chrome-linux-x64.zip"
    } else {
        "clawser-chrome-macos-x64.zip"
    }
}

fn chrome_exe_name() -> &'static str {
    if cfg!(windows) { "chrome.exe" } else { "chrome" }
}

/// Pick a free TCP port.
pub(crate) fn pick_free_port() -> io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Spawn chrome.exe with CDP and antidetect config.
pub(crate) async fn spawn_chrome(
    headless: bool,
    cdp_port: u16,
    config_path: &str,
    profile_id: Option<&str>,
) -> io::Result<Child> {
    let exe_path = find_chrome_exe().await?;
    let exe_dir = exe_path.parent().unwrap_or(exe_path.as_ref());

    let mut cmd = Command::new(&exe_path);
    cmd.current_dir(exe_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .arg(format!("--remote-debugging-port={}", cdp_port))
        .arg("--remote-allow-origins=*")
        .arg(format!("--clawser-config={}", config_path))
        .arg("--no-first-run")
        .arg("--disable-default-apps")
        .arg("--disable-extensions")
        .arg("--disable-sync")
        .arg("--no-sandbox");

    if headless {
        cmd.arg("--headless=new");
    }

    // Stable user-data-dir per profile = cookies/localStorage persist across sessions.
    // Stored at <crate_root>/.clawser/profiles/ — next to Cargo.toml of the consuming project.
    // CARGO_MANIFEST_DIR is set by cargo at runtime for examples/tests,
    // falls back to current_dir for release binaries.
    let project_root = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let user_data = match profile_id {
        Some(id) => project_root.join(".clawser").join("profiles").join(id),
        None => env::temp_dir().join(format!("clawser-{}", cdp_port)),
    };
    cmd.arg(format!("--user-data-dir={}", user_data.display()));

    #[cfg(windows)]
    {
        cmd.creation_flags(0);
    }

    cmd.spawn().map_err(|e| {
        io::Error::other(format!("Failed to spawn {}: {}", exe_path.display(), e))
    })
}

/// Wait for CDP to be ready by polling /json/version.
pub(crate) async fn wait_for_cdp(port: u16, timeout: Duration) -> io::Result<()> {
    let url = format!("http://127.0.0.1:{}/json/version", port);
    let start = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| io::Error::other(format!("reqwest client failed: {}", e)))?;

    loop {
        if start.elapsed() > timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("CDP not ready after {:?} on port {}", timeout, port),
            ));
        }
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            _ => tokio::time::sleep(Duration::from_millis(300)).await,
        }
    }
}

/// Get the first page's WebSocket debugger URL.
pub(crate) async fn get_page_ws_url(port: u16) -> io::Result<String> {
    let url = format!("http://127.0.0.1:{}/json", port);
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| io::Error::other(format!("CDP /json failed: {}", e)))?;
    let body = resp
        .text()
        .await
        .map_err(|e| io::Error::other(format!("CDP /json read failed: {}", e)))?;
    let tabs: Vec<serde_json::Value> = serde_json::from_str(&body)
        .map_err(|e| io::Error::other(format!("CDP /json parse failed: {}", e)))?;

    for tab in &tabs {
        if let Some(ws_url) = tab.get("webSocketDebuggerUrl").and_then(|v| v.as_str()) {
            return Ok(ws_url.to_string());
        }
    }

    Err(io::Error::other("No page with webSocketDebuggerUrl found"))
}
