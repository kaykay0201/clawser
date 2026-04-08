//! Minimal spawn test — try different creation flags.
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::fs::File;

fn try_spawn(label: &str, flags: u32) -> bool {
    let exe = std::env::var("CLAWSER_BROWSER_PATH")
        .unwrap_or_else(|_| "clawser_browser.exe".to_string());
    let exe_path = std::path::Path::new(&exe);
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));

    eprintln!("[{}] spawning with creation_flags={:#x}...", label, flags);

    let stderr_file = File::create(exe_dir.join("spawn_test_stderr.log")).ok();

    #[cfg(windows)]
    let child_result = {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new(&exe);
        cmd.current_dir(exe_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        if let Some(f) = stderr_file {
            cmd.stderr(Stdio::from(f));
        } else {
            cmd.stderr(Stdio::inherit());
        }
        cmd.creation_flags(flags);
        cmd.spawn()
    };
    #[cfg(not(windows))]
    let child_result = Command::new(&exe)
        .current_dir(exe_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn();

    let mut child = match child_result {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[{}] spawn failed: {}", label, e);
            return false;
        }
    };

    eprintln!("[{}] pid={}", label, child.id());

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);

    // Small delay for process startup
    std::thread::sleep(std::time::Duration::from_millis(500));

    let cmd = r#"{"id":0,"cmd":"init"}"#;
    eprintln!("[{}] sending init...", label);
    if writeln!(stdin, "{}", cmd).is_err() {
        eprintln!("[{}] write failed", label);
        let _ = child.kill();
        return false;
    }
    let _ = stdin.flush();

    eprintln!("[{}] reading response...", label);
    let mut line = String::new();
    let n = reader.read_line(&mut line).unwrap_or(0);
    eprintln!("[{}] got {} bytes: {}", label, n, line.trim());

    if n == 0 {
        eprintln!("[{}] FAILED — EOF", label);
        // Print stderr log
        let log_path = exe_path.parent().unwrap().join("spawn_test_stderr.log");
        if let Ok(content) = std::fs::read_to_string(&log_path) {
            let preview: String = content.chars().take(500).collect();
            eprintln!("[{}] stderr: {}", label, preview);
        }
        let _ = child.kill();
        let _ = child.wait();
        return false;
    }

    // Shutdown
    let _ = writeln!(stdin, r#"{{"id":1,"cmd":"shutdown"}}"#);
    let _ = stdin.flush();
    let mut line2 = String::new();
    let _ = reader.read_line(&mut line2);
    let status = child.wait().unwrap();
    eprintln!("[{}] PASSED (exit {})", label, status);
    true
}

fn main() {
    let flags_to_try: Vec<(&str, u32)> = vec![
        ("flags=0", 0),
        ("CREATE_NEW_CONSOLE", 0x10),
        ("CREATE_NO_WINDOW", 0x08000000),
        ("DETACHED_PROCESS", 0x08),
    ];

    for (label, flags) in &flags_to_try {
        if try_spawn(label, *flags) {
            eprintln!("\n=== {} WORKS ===", label);
            return;
        }
        eprintln!("");
    }

    eprintln!("\n=== ALL FLAGS FAILED ===");
    std::process::exit(1);
}
