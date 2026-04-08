use std::env;
use std::io::{self, BufReader};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::types::Seed;

/// Locate the clawser_browser executable.
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

/// Spawn the browser subprocess with command-line config.
/// The browser initializes fully (antidetect, headless/headful, etc.)
/// and writes a ready signal to stdout. We wait for that before returning.
pub(crate) fn spawn_browser(
    headless: bool,
    seed: Option<&Seed>,
    watch: &[String],
) -> io::Result<(Child, ChildStdin, BufReader<ChildStdout>, Seed)> {
    let exe_path = find_browser_exe()?;
    let exe_dir = exe_path.parent().unwrap_or(exe_path.as_ref());

    // Redirect stderr to a log file — Chromium's RouteStdioToConsole
    // corrupts the CRT heap when stderr is inherited/piped on Windows.
    let stderr_path = exe_dir.join("clawser_browser_stderr.log");
    let stderr_file = std::fs::File::create(&stderr_path)
        .unwrap_or_else(|_| std::fs::File::create("/dev/null").unwrap());

    let mut cmd = Command::new(&exe_path);
    cmd.current_dir(exe_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(stderr_file));

    // Pass config via command line args
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
    cmd.creation_flags(0);

    let mut child = cmd.spawn().map_err(|e| {
        io::Error::other(format!("Failed to spawn {}: {}", exe_path.display(), e))
    })?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("Failed to capture browser stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Failed to capture browser stdout"))?;
    let mut reader = BufReader::new(stdout);

    // Wait for the ready signal — browser is fully initialized.
    let ready = crate::protocol::read_ready(&mut reader)?;
    let seed_val = ready
        .get("seed")
        .ok_or_else(|| io::Error::other("ready signal missing 'seed'"))?;
    let seed: Seed = serde_json::from_value(seed_val.clone())
        .map_err(|e| io::Error::other(format!("failed to parse seed: {}", e)))?;

    Ok((child, stdin, reader, seed))
}
