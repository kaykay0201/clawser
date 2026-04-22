//! Batch test — 3 parallel headful browsers × 10 rounds with proxy rotation.
//! Rotate IP (5s cooldown) → launch 3 browsers → browsers live 30s → close → repeat.

use clawser_browser::Browser;
use std::time::Instant;

const PER_ROUND: usize = 3;
const ROUNDS: usize = 10;
const BROWSER_LIVE_SECS: u64 = 30;
const ROTATE_COOLDOWN_SECS: u64 = 5;

const PROXY_KEY: &str = "h9f015c1zpu65df90452b6349a63a9ac2d7bf15be1727e453ab6013";

const USERS: [&str; 30] = [
    "chokevy", "phanminhtu", "lehoangnam", "nguyenvana",
    "tranthimai", "vothanhdat", "buiquochuy", "dangtuanhung",
    "lythihuong", "hoanganhduc", "ngothikieu", "phamduclong",
    "truongmylinh", "doanthanhson", "luuquanghai", "nguyenthilan",
    "vutrongnhan", "lequangvinh", "tranductho", "phamthingoc",
    "hothienphuc", "nguyenkhoa", "vuthanhtrung", "buithihoa",
    "dangquocbao", "lethimynhi", "phanducminh", "truongankhang",
    "nguyenhaiyen", "vothianhthy",
];

fn api_url() -> String {
    std::env::var("GAME_API_URL").unwrap_or_else(|_| "https://api.abb1211.com/endpoint/play".into())
}
fn api_token() -> String {
    std::env::var("GAME_API_TOKEN").expect("Set GAME_API_TOKEN in .env")
}

fn load_dotenv() {
    for path in ["clawser-browser/.env", ".env"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some((k, v)) = line.split_once('=') {
                    std::env::set_var(k.trim(), v.trim());
                }
            }
            break;
        }
    }
}

async fn rotate_and_get_proxy() -> Option<String> {
    let client = reqwest::Client::new();

    // Rotate
    let url = format!("https://api.zingproxy.com/open/change-ip/{}", PROXY_KEY);
    match client.get(&url).send().await {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let msg = body["message"].as_str().unwrap_or("?");
            println!("  Rotate: {msg}");
        }
        Err(e) => { println!("  Rotate FAIL: {e}"); return None; }
    }

    tokio::time::sleep(std::time::Duration::from_secs(ROTATE_COOLDOWN_SECS)).await;

    // Get proxy — re-fetch to get new IP after rotation
    let url = format!("https://api.zingproxy.com/open/get-proxy/{}", PROXY_KEY);
    match client.get(&url).send().await {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            // Try HTTP proxy first (more compatible), fallback to SOCKS5
            if let Some(http) = body["proxy"]["httpProxy"].as_str() {
                let parts: Vec<&str> = http.splitn(4, ':').collect();
                if parts.len() >= 2 {
                    let proxy = format!("http://{}:{}", parts[0], parts[1]);
                    println!("  Proxy: {proxy}");
                    return Some(proxy);
                }
            }
            println!("  Get proxy FAIL: {body}");
            None
        }
        Err(e) => { println!("  Get proxy FAIL: {e}"); None }
    }
}

/// Verify proxy works before launching browsers.
async fn verify_proxy(proxy: &str) -> bool {
    let proxy_obj = match reqwest::Proxy::all(proxy) {
        Ok(p) => p,
        Err(e) => { println!("  Verify FAIL (bad proxy url): {e}"); return false; }
    };
    let client = match reqwest::Client::builder()
        .proxy(proxy_obj)
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => { println!("  Verify FAIL (client): {e}"); return false; }
    };
    // Use http (not https) to avoid TLS-over-proxy issues during verify
    match client.get("http://httpbin.org/ip").send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            println!("  Verify OK: {}", body.trim());
            true
        }
        Err(e) => { println!("  Verify FAIL: {e}"); false }
    }
}

#[tokio::main]
async fn main() {
    load_dotenv();
    let total = PER_ROUND * ROUNDS;
    println!("=== Proxy batch: {}×{} = {} | live {}s | rotate {}s cooldown ===\n",
        PER_ROUND, ROUNDS, total, BROWSER_LIVE_SECS, ROTATE_COOLDOWN_SECS);

    let start = Instant::now();
    let mut ok = 0u32;
    let mut fail = 0u32;

    for round in 0..ROUNDS {
        println!("--- Round {}/{} ---", round + 1, ROUNDS);

        // 1. Rotate IP and get proxy
        let proxy = match rotate_and_get_proxy().await {
            Some(p) => p,
            None => {
                println!("  Skip round — no proxy");
                fail += PER_ROUND as u32;
                continue;
            }
        };

        // 2. Verify proxy works
        if !verify_proxy(&proxy).await {
            println!("  Skip round — proxy dead");
            fail += PER_ROUND as u32;
            continue;
        }

        // 3. Launch 3 browsers in parallel
        let offset = round * PER_ROUND;
        let mut handles = Vec::new();

        for i in 0..PER_ROUND {
            let idx = offset + i;
            let user_id = USERS[idx % USERS.len()].to_string();
            let proxy = proxy.clone();
            let api = api_url();
            let token = api_token();

            handles.push(tokio::spawn(async move {
                let label = format!("[{:02}/{}]", idx + 1, total);
                let t = Instant::now();

                // Get game URL (direct, no proxy)
                let game_url = match reqwest::Client::new()
                    .post(&api)
                    .header("Authorization", format!("Bearer {}", token))
                    .header("Content-Type", "application/json")
                    .body(format!(r#"{{"user_id": "{}"}}"#, user_id))
                    .send().await
                {
                    Ok(resp) => {
                        let body: serde_json::Value = resp.json().await.unwrap_or_default();
                        match body["url"].as_str() {
                            Some(u) if !u.is_empty() => u.to_string(),
                            _ => return format!("{label} FAIL api: {body}"),
                        }
                    }
                    Err(e) => return format!("{label} FAIL api: {e}"),
                };
                let api_ms = t.elapsed().as_millis();

                // Launch browser with proxy
                let browser = match Browser::builder()
                    .headful()
                    .random()
                    .proxy(&proxy)
                    .build().await
                {
                    Ok(b) => b,
                    Err(e) => return format!("{label} FAIL launch: {e}"),
                };
                let launch_ms = t.elapsed().as_millis() - api_ms;

                // Navigate (human simulation auto-starts with aggressive scroll)
                let page = match browser.new_page("about:blank").await {
                    Ok(p) => p,
                    Err(e) => return format!("{label} FAIL page: {e}"),
                };

                // Aggressive scrolling in background
                let scroll_page = page.cdp().clone();
                let scroll_handle = tokio::spawn(async move {
                    use chromiumoxide::cdp::browser_protocol::input::{
                        DispatchMouseEventParams, DispatchMouseEventType,
                    };
                    let mut y = 300.0f64;
                    loop {
                        // Scroll down aggressively
                        let mut ev = DispatchMouseEventParams::new(
                            DispatchMouseEventType::MouseWheel, 500.0, y,
                        );
                        ev.delta_x = Some(0.0);
                        ev.delta_y = Some(300.0);
                        let _ = scroll_page.execute(ev).await;
                        y = ((y + 50.0) % 800.0) + 100.0;
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    }
                });

                let t_nav = Instant::now();
                if let Err(e) = page.goto(&game_url).await {
                    scroll_handle.abort();
                    return format!("{label} FAIL nav: {e}");
                }
                let nav_ms = t_nav.elapsed().as_millis();

                // Wait for cookies
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let cookies = page.cdp().get_cookies().await.unwrap_or_default();
                let has_evo = cookies.iter().any(|c| c.name == "EVOSESSIONID");
                let evo = cookies.iter().find(|c| c.name == "EVOSESSIONID")
                    .map(|c| c.value.clone())
                    .unwrap_or_else(|| "N/A".into());
                let cookie_names: Vec<String> = cookies.iter().map(|c| c.name.clone()).collect();

                // Stay alive for remaining time (human sim + aggressive scroll running)
                let elapsed = t.elapsed().as_secs();
                if elapsed < BROWSER_LIVE_SECS {
                    tokio::time::sleep(std::time::Duration::from_secs(BROWSER_LIVE_SECS - elapsed)).await;
                }

                // Close browser
                scroll_handle.abort();
                let _ = browser.close().await;
                let total_ms = t.elapsed().as_millis();

                let status = if has_evo { "OK" } else { "MISS" };
                format!("{label} {status} | {user_id:<18} | total:{total_ms}ms api:{api_ms}ms launch:{launch_ms}ms nav:{nav_ms}ms | cookies:{} | EVO:{evo}\n       {:?}",
                    cookies.len(), cookie_names)
            }));
        }

        for h in handles {
            match h.await {
                Ok(msg) => {
                    if msg.contains("] OK ") { ok += 1; } else { fail += 1; }
                    println!("{msg}");
                }
                Err(e) => { println!("  task panic: {e}"); fail += 1; }
            }
        }

        println!();
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!("=== OK:{ok} FAIL:{fail} | {elapsed:.0}s total | {:.1}s/profile ===",
        elapsed / total as f64);
}
