//! WebSocket detection — connect to iframe + service worker targets directly.
//!
//! Run: CLAWSER_CHROME_PATH=out/Release/chrome.exe cargo run --release --manifest-path clawser-browser/Cargo.toml --example ws_watch_test

use clawser_browser::Browser;
use futures_util::StreamExt;

const API_URL: &str = "https://api.abb1211.com/endpoint/play";
const API_TOKEN: &str = "REDACTED";

#[tokio::main]
async fn main() {
    println!("=== WS Detection Test ===\n");

    let client = reqwest::Client::new();
    let resp = client.post(API_URL)
        .header("Authorization", format!("Bearer {}", API_TOKEN))
        .header("Content-Type", "application/json")
        .body(r#"{"user_id": "testuser01"}"#)
        .send().await.expect("API failed");
    let body: serde_json::Value = resp.json().await.expect("parse failed");
    let url = body["url"].as_str().expect("no url");

    let browser = Browser::builder()
        .headful()
        .profile(7, 777)
        .build().await.expect("browser failed");

    let page = browser.navigate(url).await.expect("navigate failed");
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    // List all targets
    let port = browser.cdp_port();
    let targets_url = format!("http://127.0.0.1:{}/json", port);
    let targets: Vec<serde_json::Value> = reqwest::get(&targets_url).await
        .unwrap().json().await.unwrap_or_default();

    println!("[1] Targets:");
    for t in &targets {
        let ttype = t["type"].as_str().unwrap_or("?");
        let turl = t["url"].as_str().unwrap_or("?");
        let ws_url = t["webSocketDebuggerUrl"].as_str().unwrap_or("");
        println!("    {} — {} ", ttype, &turl[..turl.len().min(80)]);
        if !ws_url.is_empty() {
            println!("      WS: {}", ws_url);
        }
    }

    // Connect to EACH target, enable Network, listen for WS events
    println!("\n[2] Scanning each target for WebSocket...");
    for t in &targets {
        let ttype = t["type"].as_str().unwrap_or("?");
        let ws_url = t["webSocketDebuggerUrl"].as_str().unwrap_or("");
        if ws_url.is_empty() { continue; }

        let mut target_ws = match tokio_tungstenite::connect_async(ws_url).await {
            Ok((ws, _)) => ws,
            Err(e) => { println!("    [{ttype}] connect failed: {e}"); continue; }
        };

        // Enable Network
        let enable_msg = serde_json::json!({"id":1,"method":"Network.enable","params":{}});
        use futures_util::SinkExt;
        let _ = target_ws.send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&enable_msg).unwrap().into()
        )).await;

        // Read events for 3 seconds
        let mut ws_found = vec![];
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            match tokio::time::timeout_at(deadline, target_ws.next()).await {
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text)))) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                        let method = parsed.get("method").and_then(|v| v.as_str()).unwrap_or("");
                        if method == "Network.webSocketCreated" {
                            let url = parsed["params"]["url"].as_str().unwrap_or("?");
                            ws_found.push(url.to_string());
                        }
                    }
                }
                Err(_) => break, // timeout
                _ => {}
            }
        }

        if ws_found.is_empty() {
            println!("    [{ttype}] no WebSocket found");
        } else {
            println!("    [{ttype}] FOUND {} WebSocket(s):", ws_found.len());
            for u in &ws_found {
                println!("      {}", &u[..u.len().min(120)]);
            }
        }
    }

    println!("\nBrowser open. Ctrl+C to exit.");
    loop { tokio::time::sleep(std::time::Duration::from_secs(60)).await; }
}
