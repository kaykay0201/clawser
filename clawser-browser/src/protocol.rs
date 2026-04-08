use serde_json::Value;
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Send a JSON command to the browser process via TCP.
pub(crate) fn send_command(
    writer: &mut impl Write,
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
    writeln!(writer, "{}", line)?;
    writer.flush()?;
    Ok(id)
}

/// Wait for the ready signal from the browser process.
pub(crate) fn read_ready(
    reader: &mut impl BufRead,
) -> io::Result<Value> {
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line)?;
    if bytes_read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "browser closed connection before ready signal",
        ));
    }

    let parsed: Value =
        serde_json::from_str(line.trim()).map_err(|e| io::Error::other(e))?;

    if !parsed.get("ready").and_then(|v| v.as_bool()).unwrap_or(false) {
        return Err(io::Error::other(format!(
            "expected ready signal, got: {}",
            line.trim()
        )));
    }

    Ok(parsed)
}

/// Read a JSON response line. Blocks until a line is available.
pub(crate) fn read_response(
    reader: &mut impl BufRead,
) -> io::Result<Value> {
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line)?;
    if bytes_read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "browser closed connection",
        ));
    }

    let parsed: Value =
        serde_json::from_str(line.trim()).map_err(|e| io::Error::other(e))?;

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
