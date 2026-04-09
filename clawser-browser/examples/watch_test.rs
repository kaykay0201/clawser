//! Watch + capture + replay test with Evo Games.
//!
//! Run: CLAWSER_CHROME_PATH=out/Release/chrome.exe cargo run --release --manifest-path clawser-browser/Cargo.toml --example watch_test

use clawser_browser::Browser;
use std::time::Instant;

const API_URL: &str = "https://api.abb1211.com/endpoint/play";
const API_TOKEN: &str = "REDACTED";

#[tokio::main]
async fn main() {
    println!("=== Watch + Capture + Replay Test ===\n");

    // 1. Get game URL
    let client = reqwest::Client::new();
    let resp = client
        .post(API_URL)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", API_TOKEN))
        .header("Content-Type", "application/json")
        .body(r#"{"user_id": "beezsbee"}"#)
        .send().await.expect("API failed");
    let body: serde_json::Value = resp.json().await.expect("parse failed");
    let url = body["url"].as_str().expect("no url");
    println!("[1] Got game URL");

    // 2. Launch browser WITH watch patterns — registered BEFORE page load
    let t0 = Instant::now();
    let browser = Browser::builder()
        .headful()
        .profile(7, 777)
        .watch(&["/api/", "/setup", "/config", "/game", "/lobby"])
        .build().await
        .expect("browser failed");
    println!("[2] Browser ready ({:?})", t0.elapsed());

    // 3. Navigate — hooks are already injected, will capture matching requests
    let page = browser.navigate(url).await.expect("navigate failed");
    println!("[3] Navigating...");

    // 4. Quick behavior
    let _ = page.mouse_move(400.0, 300.0, 5).await;

    // 5. Wait for page to load + captures to accumulate
    println!("[4] Waiting for captures...");
    tokio::time::sleep(std::time::Duration::from_secs(8)).await;

    // 6. Show all captures
    let captures = page.captures().await.unwrap_or_default();
    println!("\n[5] Captured {} requests:", captures.len());
    for (i, cap) in captures.iter().enumerate() {
        println!("    [{}] {} {} → {} ({} bytes)",
            i, cap.method, &cap.url[..cap.url.len().min(80)],
            cap.status, cap.response_body.len());
    }

    // 7. Replay first capture if any
    if let Some(first) = captures.first() {
        println!("\n[6] Replaying: {} {}...", first.method, &first.url[..first.url.len().min(60)]);

        let t = Instant::now();
        let r1 = page.replay(first).await.expect("replay failed");
        println!("    Replay 1: {} bytes ({:?})", r1.len(), t.elapsed());

        let t = Instant::now();
        let r2 = page.replay(first).await.expect("replay failed");
        println!("    Replay 2: {} bytes ({:?})", r2.len(), t.elapsed());

        let t = Instant::now();
        let r3 = page.replay(first).await.expect("replay failed");
        println!("    Replay 3: {} bytes ({:?})", r3.len(), t.elapsed());
    } else {
        println!("\n[6] No captures to replay. Try adding more watch patterns.");
    }

    println!("\n    Browser stays open. Press Ctrl+C to exit.");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
