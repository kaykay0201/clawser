use clawser_browser::Browser;
use chromiumoxide::cdp::browser_protocol::network::{
    EnableParams, EventWebSocketCreated, EventWebSocketClosed,
};
use futures_util::StreamExt;

const API_URL: &str = "https://api.abb1211.com/endpoint/play";
const API_TOKEN: &str = "REDACTED";

#[tokio::main]
async fn main() {
    println!("=== Game Test + WS Detection ===\n");

    // 1. Fetch game URL
    println!("Fetching game URL...");
    let client = reqwest::Client::new();
    let resp = client
        .post(API_URL)
        .header("Authorization", format!("Bearer {API_TOKEN}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(r#"{"user_id": "beezsbee"}"#)
        .send()
        .await
        .expect("API request failed");

    let body: serde_json::Value = resp.json().await.expect("JSON parse failed");
    let game_url = body["url"].as_str().expect("no 'url' in response");
    println!("Game URL: {game_url}\n");

    // 2. Launch browser
    let browser = Browser::builder()
        .headful()
        .profile(42, 12345)
        .build()
        .await
        .expect("browser launch failed");

    // 3. Open blank page first so we can set up listeners before navigation
    let page = browser.new_page("about:blank").await.expect("page failed");

    // Enable network domain for WS events
    page.cdp().execute(EnableParams::default()).await.expect("network enable failed");

    // Set up WS event listeners
    let mut ws_created = page.cdp().event_listener::<EventWebSocketCreated>().await.expect("listener failed");
    let mut ws_closed = page.cdp().event_listener::<EventWebSocketClosed>().await.expect("listener failed");

    // Spawn listener tasks
    tokio::spawn(async move {
        while let Some(ev) = ws_created.next().await {
            println!("[WS CREATED] request_id={:?} url={}", ev.request_id, ev.url);
        }
    });

    tokio::spawn(async move {
        while let Some(ev) = ws_closed.next().await {
            println!("[WS CLOSED]  request_id={:?}", ev.request_id);
        }
    });

    // 4. Navigate to game
    println!("Navigating to game...");
    page.navigate(game_url).await.expect("nav failed");

    // 5. Wait and monitor
    println!("Watching for WebSocket connections...\n");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
