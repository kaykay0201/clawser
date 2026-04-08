//! Minimal spawn test — bypass the library, test pipe directly.
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn main() {
    let exe = std::env::var("CLAWSER_BROWSER_PATH")
        .unwrap_or_else(|_| "clawser_browser.exe".to_string());
    let exe_path = std::path::Path::new(&exe);
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));

    eprintln!("[spawn_test] exe={}", exe);
    eprintln!("[spawn_test] dir={}", exe_dir.display());

    #[cfg(windows)]
    let mut child = {
        use std::os::windows::process::CommandExt;
        Command::new(&exe)
            .current_dir(exe_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .creation_flags(0)
            .spawn()
            .expect("spawn failed")
    };
    #[cfg(not(windows))]
    let mut child = Command::new(&exe)
        .current_dir(exe_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn failed");

    eprintln!("[spawn_test] pid={}", child.id());

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // Send init
    let cmd = r#"{"id":0,"cmd":"init"}"#;
    eprintln!("[spawn_test] sending: {}", cmd);
    writeln!(stdin, "{}", cmd).expect("write failed");
    stdin.flush().expect("flush failed");

    // Read response
    eprintln!("[spawn_test] reading...");
    let mut line = String::new();
    let n = reader.read_line(&mut line).expect("read failed");
    eprintln!("[spawn_test] got {} bytes: {}", n, line.trim());

    if n == 0 {
        eprintln!("[spawn_test] EOF! Child status:");
        match child.try_wait() {
            Ok(Some(s)) => eprintln!("  exited: {}", s),
            Ok(None) => eprintln!("  still running"),
            Err(e) => eprintln!("  error: {}", e),
        }
        std::process::exit(1);
    }

    // Shutdown
    writeln!(stdin, r#"{{"id":1,"cmd":"shutdown"}}"#).unwrap();
    stdin.flush().unwrap();
    let mut line2 = String::new();
    let _ = reader.read_line(&mut line2);

    let status = child.wait().expect("wait failed");
    eprintln!("[spawn_test] child exited: {}", status);
    eprintln!("[spawn_test] PASSED");
}
