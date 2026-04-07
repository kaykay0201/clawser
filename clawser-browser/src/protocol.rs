use serde_json::Value;
use std::io::{self, BufRead, BufReader, Write};
use std::process::{ChildStdin, ChildStdout};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Send a JSON command to the browser process via stdin.
pub(crate) fn send_command(
    stdin: &mut ChildStdin,
    cmd: &str,
    params: Value,
) -> io::Result<u64> {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let msg = match params {
        Value::Object(mut map) => {
            map.insert("id".to_string(), Value::from(id));
            map.insert("cmd".to_string(), Value::from(cmd));
            Value::Object(map)
        }
        _ => {
            let mut map = serde_json::Map::new();
            map.insert("id".to_string(), Value::from(id));
            map.insert("cmd".to_string(), Value::from(cmd));
            Value::Object(map)
        }
    };

    let line = serde_json::to_string(&msg).map_err(|e| io::Error::other(e))?;
    writeln!(stdin, "{}", line)?;
    stdin.flush()?;
    Ok(id)
}

/// Send the init command (id=0, special case).
pub(crate) fn send_init(
    stdin: &mut ChildStdin,
    seed: Option<&crate::types::Seed>,
    watch: &[String],
) -> io::Result<()> {
    let mut map = serde_json::Map::new();
    map.insert("id".to_string(), Value::from(0));
    map.insert("cmd".to_string(), Value::from("init"));

    if let Some(s) = seed {
        map.insert("seed".to_string(), serde_json::to_value(s).unwrap());
    }

    if !watch.is_empty() {
        let watch_arr: Vec<Value> = watch.iter().map(|w| Value::from(w.as_str())).collect();
        map.insert("watch".to_string(), Value::Array(watch_arr));
    }

    let line = serde_json::to_string(&Value::Object(map))
        .map_err(|e| io::Error::other(e))?;
    writeln!(stdin, "{}", line)?;
    stdin.flush()?;
    Ok(())
}

/// Read a JSON response line from stdout. Blocks until a line is available.
pub(crate) fn read_response(
    stdout: &mut BufReader<ChildStdout>,
) -> io::Result<Value> {
    let mut line = String::new();
    let bytes_read = stdout.read_line(&mut line)?;
    if bytes_read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "browser process closed stdout",
        ));
    }

    let parsed: Value =
        serde_json::from_str(line.trim()).map_err(|e| io::Error::other(e))?;

    // Check for error response
    if let Some(ok) = parsed.get("ok") {
        if !ok.as_bool().unwrap_or(false) {
            let error_msg = parsed
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            return Err(io::Error::other(format!("browser error: {}", error_msg)));
        }
    }

    Ok(parsed)
}
