use std::env;
use std::io;
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::Duration;

use tokio::process::{Child, Command};

/// Find chrome.exe — checks CLAWSER_CHROME_PATH, then exe-relative, then out/Default.
pub(crate) fn find_chrome_exe() -> io::Result<PathBuf> {
    if let Ok(path) = env::var("CLAWSER_CHROME_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }

    if let Ok(exe) = env::current_exe() {
        let dir = exe.parent().unwrap_or(exe.as_ref());
        let candidate = dir.join(chrome_exe_name());
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    let candidate = PathBuf::from("out/Default").join(chrome_exe_name());
    if candidate.exists() {
        return Ok(candidate);
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Cannot find {}. Set CLAWSER_CHROME_PATH env var.", chrome_exe_name()),
    ))
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
pub(crate) fn spawn_chrome(
    headless: bool,
    cdp_port: u16,
    config_path: &str,
    profile_id: Option<&str>,
) -> io::Result<Child> {
    let exe_path = find_chrome_exe()?;
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

    // Stable user-data-dir per profile = cookies persist across sessions.
    let user_data = match profile_id {
        Some(id) => env::temp_dir().join(format!("clawser-profile-{}", id)),
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
