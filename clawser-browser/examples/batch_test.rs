//! Fast batch — skip CreepJS, go straight to game. 5 parallel × 3 rounds.

use clawser_browser::Browser;
use std::time::Instant;

const API_URL: &str = "https://api.abb1211.com/endpoint/play";
const API_TOKEN: &str = "REDACTED";
const PER_ROUND: usize = 5;
const ROUNDS: usize = 3;

async fn run_profile(idx: usize) -> (usize, String) {
    let label = format!("[P{:02}]", idx);
    let start = Instant::now();
    let user_id = format!("player_{:05}", 40000 + idx);

    // 1. Get game URL
    let game_url = match reqwest::Client::new()
        .post(API_URL)
        .header("Authorization", format!("Bearer {API_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"user_id": "{}"}}"#, user_id))
        .send().await
        .and_then(|r| Ok(r))
    {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            match body["url"].as_str() {
                Some(u) if !u.is_empty() => u.to_string(),
                _ => return (idx, format!("{label} FAIL: no game URL")),
            }
        }
        Err(e) => return (idx, format!("{label} FAIL: API {e}")),
    };

    // 2. Launch + navigate directly
    let browser = match Browser::builder().random().build().await {
        Ok(b) => b,
        Err(e) => return (idx, format!("{label} FAIL: launch {e}")),
    };

    let page = match browser.new_page(&game_url).await {
        Ok(p) => p,
        Err(e) => return (idx, format!("{label} FAIL: nav {e}")),
    };

    tokio::time::sleep(std::time::Duration::from_secs(6)).await;

    let title = page.js("document.title").await.unwrap_or_default();
    let path = format!("clawser-browser/game_{:02}.png", idx);
    if let Ok(png) = page.screenshot().await {
        let _ = std::fs::write(&path, &png);
    }

    let _ = browser.close().await;

    let secs = start.elapsed().as_secs_f64();
    let blocked = title.contains("Denied");
    let status = if blocked { "BLOCKED" } else { "OK" };

    (idx, format!("{label} {status} {secs:.1}s | {}", title.chars().take(30).collect::<String>()))
}

#[tokio::main]
async fn main() {
    let total = PER_ROUND * ROUNDS;
    println!("=== {} rounds × {} = {} profiles ===\n", ROUNDS, PER_ROUND, total);
    let start = Instant::now();
    let mut ok = 0u32;
    let mut blocked = 0u32;
    let mut failed = 0u32;

    for round in 0..ROUNDS {
        println!("--- Round {}/{} ---", round + 1, ROUNDS);
        let offset = round * PER_ROUND;
        let mut handles = Vec::new();
        for i in 0..PER_ROUND {
            handles.push(tokio::spawn(run_profile(offset + i)));
        }
        for h in handles {
            if let Ok((_, msg)) = h.await {
                if msg.contains("] OK ") { ok += 1; }
                else if msg.contains("BLOCKED") { blocked += 1; }
                else { failed += 1; }
                println!("{msg}");
            }
        }
        println!();
    }

    println!("=== Total: {} | OK: {} | Blocked: {} | Failed: {} | Rate: {:.0}% | {:.0}s ===",
        total, ok, blocked, failed, blocked as f64 / total as f64 * 100.0, start.elapsed().as_secs_f64());
}
