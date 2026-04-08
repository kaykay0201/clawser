use std::env;
use std::io::{self, BufReader};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use crate::types::Seed;

fn find_browser_exe() -> io::Result<PathBuf> {
    if let Ok(path) = env::var("CLAWSER_BROWSER_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    if let Ok(exe) = env::current_exe() {
        let dir = exe.parent().unwrap_or(exe.as_ref());
        let candidate = dir.join(exe_name());
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    let candidate = PathBuf::from(exe_name());
    if candidate.exists() {
        return Ok(candidate);
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!(
            "Cannot find {}. Set CLAWSER_BROWSER_PATH or place it next to your executable.",
            exe_name()
        ),
    ))
}

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "clawser_browser.exe"
    } else {
        "clawser_browser"
    }
}

/// Pick a free TCP port by binding to :0, getting the port, then closing.
fn pick_free_port() -> io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

/// Spawn the browser process with TCP communication.
///
/// 1. Pick a free port
/// 2. Spawn clawser_browser.exe --clawser-port=PORT [--headless] [--clawser-seed=...]
/// 3. Connect to 127.0.0.1:PORT (retry until browser is ready)
/// 4. Read the ready signal {"ready":true,"seed":{...}}
/// 5. Return the connection for command I/O
pub(crate) fn spawn_browser(
    headless: bool,
    seed: Option<&Seed>,
    watch: &[String],
) -> io::Result<(Child, BufReader<TcpStream>, TcpStream, Seed)> {
    let exe_path = find_browser_exe()?;
    let exe_dir = exe_path.parent().unwrap_or(exe_path.as_ref());

    let port = pick_free_port()?;

    let mut cmd = Command::new(&exe_path);
    cmd.current_dir(exe_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg(format!("--clawser-port={}", port));

    if headless {
        cmd.arg("--headless");
    }
    if let Some(s) = seed {
        cmd.arg(format!(
            "--clawser-seed={},{},{},{},{}",
            s.hw_seed, s.canvas_seed, s.webgl_seed, s.audio_seed, s.client_rects_seed
        ));
    }
    if !watch.is_empty() {
        cmd.arg(format!("--clawser-watch={}", watch.join(",")));
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0);
    }

    let child = cmd.spawn().map_err(|e| {
        io::Error::other(format!("Failed to spawn {}: {}", exe_path.display(), e))
    })?;

    // Connect to the browser's TCP listener. Retry for up to 30 seconds
    // while the browser initializes Chromium + antidetect.
    let addr = format!("127.0.0.1:{}", port);
    let mut stream = None;
    for attempt in 0..60 {
        match TcpStream::connect(&addr) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(_) => {
                std::thread::sleep(Duration::from_millis(500));
                if attempt > 0 && attempt % 10 == 0 {
                    eprintln!(
                        "[clawser-browser] Waiting for browser TCP on port {}... ({}s)",
                        port,
                        attempt / 2
                    );
                }
            }
        }
    }

    let stream = stream.ok_or_else(|| {
        io::Error::other(format!(
            "Timed out connecting to browser on 127.0.0.1:{}",
            port
        ))
    })?;

    // Set read timeout for the ready signal.
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let writer = stream;

    // Read the ready signal.
    let ready = crate::protocol::read_ready(&mut reader)?;
    let seed_val = ready
        .get("seed")
        .ok_or_else(|| io::Error::other("ready signal missing 'seed'"))?;
    let seed: Seed = serde_json::from_value(seed_val.clone())
        .map_err(|e| io::Error::other(format!("failed to parse seed: {}", e)))?;

    // Clear read timeout for normal operation (commands may block).
    writer.set_read_timeout(None)?;

    Ok((child, reader, writer, seed))
}
