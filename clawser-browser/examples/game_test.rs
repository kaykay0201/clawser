//! Game test: get link → load + behavior simultaneously → wait for EVOSESSIONID.
//!
//! Run: CLAWSER_CHROME_PATH=out/Release/chrome.exe cargo run --release --manifest-path clawser-browser/Cargo.toml --example game_test

use clawser_browser::Browser;
use std::time::Instant;

const API_URL: &str = "https://api.abb1211.com/endpoint/play";
const API_TOKEN: &str = "REDACTED";

#[tokio::main]
async fn main() {
    println!("=== Game Test ===\n");

    // 1. Get game URL
    let client = reqwest::Client::new();
    let resp = client
        .post(API_URL)
        .header("Accept", "application/json")
        .header("Authorization", format!("Bearer {}", API_TOKEN))
        .header("Content-Type", "application/json")
        .body(r#"{"user_id": "beezsbee"}"#)
        .send()
        .await
        .expect("API failed");
    let body: serde_json::Value = resp.json().await.expect("parse failed");
    let url = body["url"].as_str().expect("no url");
    println!("[1] Got URL");

    // 2. Launch + navigate
    let t0 = Instant::now();
    // Fixed profile — reuses cookies from previous sessions
    let browser = Browser::builder().headful().profile(7, 777).build().await
        .expect("browser failed");
    println!("[2] Browser ready ({:?})", t0.elapsed());

    let page = browser.navigate(url).await.expect("navigate failed");
    println!("[3] Navigating... ({:?})", t0.elapsed());

    // 3. Behavior + poll cookies simultaneously
    //    Mouse/scroll runs between cookie checks — doesn't block, keeps Akamai happy
    let mut session_id = String::new();
    for i in 0u32..120 {
        // Quick behavior burst every few iterations
        if i % 4 == 0 {
            let x = 200.0 + ((i as f64 * 37.0) % 600.0);
            let y = 150.0 + ((i as f64 * 23.0) % 400.0);
            let _ = page.mouse_move(x, y, 5).await;
        }
        if i == 3 {
            let _ = page.scroll(150).await;
        }

        // Check for EVOSESSIONID — use empty string to get ALL cookies
        let cookies = browser.cookies("").await.unwrap_or_default();
        if let Some(c) = cookies.iter().find(|c| c.name.contains("EVOSESSIONID")) {
            session_id = c.value.clone();
            println!("[4] EVOSESSIONID found in {:?}", t0.elapsed());
            println!("    {}", session_id);
            println!("    Total cookies: {}", cookies.len());
            for c in &cookies {
                println!("    {}={}", c.name, &c.value[..c.value.len().min(50)]);
            }
            break;
        }

        if i == 119 {
            println!("[4] Timeout after {:?} — no EVOSESSIONID", t0.elapsed());
        }

        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    println!("\n=== BROWSER → SESSION: {:?} ===", t0.elapsed());
    println!("    Browser stays open. Press Ctrl+C to exit.");

    // Keep alive — don't close browser
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
