//! Fast batch — 2 parallel × 10 rounds = 20 profiles headless.

use clawser_browser::Browser;
use std::time::Instant;

const API_URL: &str = "https://api.abb1211.com/endpoint/play";
const API_TOKEN: &str = env!("GAME_API_TOKEN", "Set GAME_API_TOKEN env var");
const PER_ROUND: usize = 2;
const ROUNDS: usize = 10;

async fn run_profile(idx: usize) -> (usize, String) {
    let label = format!("[P{:02}]", idx);
    let t0 = Instant::now();
    let user_id = format!("beez{:x}", (idx as u64).wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(t0.elapsed().as_nanos() as u64));

    // 1. Get game URL
    let t_api = Instant::now();
    let game_url = match reqwest::Client::new()
        .post(API_URL)
        .header("Authorization", format!("Bearer {API_TOKEN}"))
        .header("Content-Type", "application/json")
        .body(format!(r#"{{"user_id": "{}"}}"#, user_id))
        .send().await
    {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            match body["url"].as_str() {
                Some(u) if !u.is_empty() => u.to_string(),
                _ => return (idx, format!("{label} FAIL: no game URL | body: {}", body)),
            }
        }
        Err(e) => return (idx, format!("{label} FAIL: API {e}")),
    };
    let api_ms = t_api.elapsed().as_millis();

    // 2. Launch browser
    let t_launch = Instant::now();
    let browser = match Browser::builder().random().build().await {
        Ok(b) => b,
        Err(e) => return (idx, format!("{label} FAIL: launch {e}")),
    };
    let launch_ms = t_launch.elapsed().as_millis();

    // 3. Navigate — wait for page load, no fixed sleep
    let t_nav = Instant::now();
    let page = match browser.new_page("about:blank").await {
        Ok(p) => p,
        Err(e) => return (idx, format!("{label} FAIL: page {e}")),
    };
    if let Err(e) = page.goto(&game_url).await {
        let _ = browser.close().await;
        return (idx, format!("{label} FAIL: nav {e}"));
    }
    let nav_ms = t_nav.elapsed().as_millis();

    // 4. Check title
    let title = page.js("document.title").await.unwrap_or_default();
    let blocked = title.contains("Denied");

    // 5. Capture cookies
    let t_cookie = Instant::now();
    let cookies = page.cdp().get_cookies().await.unwrap_or_default();
    let cookie_ms = t_cookie.elapsed().as_millis();

    let has_abck = cookies.iter().any(|c| c.name == "_abck");
    let has_akbmsc = cookies.iter().any(|c| c.name == "ak_bmsc");
    let total_cookies = cookies.len();
    let cookie_names: Vec<_> = cookies.iter().map(|c| c.name.as_str()).collect();

    let _ = browser.close().await;

    let total_ms = t0.elapsed().as_millis();
    let status = if blocked { "BLOCKED" } else { "OK" };

    (idx, format!(
        "{label} {status} | total:{total_ms}ms api:{api_ms}ms launch:{launch_ms}ms nav:{nav_ms}ms cookie:{cookie_ms}ms | cookies:{total_cookies} _abck:{has_abck} ak_bmsc:{has_akbmsc} | {} | {:?}",
        title.chars().take(20).collect::<String>(),
        cookie_names,
    ))
}

#[tokio::main]
async fn main() {
    let total = PER_ROUND * ROUNDS;
    println!("=== {} rounds × {} = {} profiles ===\n", ROUNDS, PER_ROUND, total);
    let start = Instant::now();
    let mut ok = 0u32;
    let mut blocked = 0u32;
    let mut failed = 0u32;
    let mut has_abck_count = 0u32;

    for round in 0..ROUNDS {
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
                if msg.contains("_abck:true") { has_abck_count += 1; }
                println!("{msg}");
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!("\n=== Total: {} | OK: {} | Blocked: {} | Failed: {} | Block: {:.0}% | _abck: {}/{} | {:.0}s | {:.1}s/profile ===",
        total, ok, blocked, failed, blocked as f64 / total as f64 * 100.0, has_abck_count, ok + blocked, elapsed, elapsed / total as f64);
}
