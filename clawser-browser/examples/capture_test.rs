//! Test page capture: MHTML (full page) + HTML (DOM only).
//!
//! Run: CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example capture_test

use clawser_browser::Browser;

#[tokio::main]
async fn main() {
    let browser = Browser::builder()
        .headful()
        .random()
        .build().await
        .expect("failed to create browser");

    let page = browser.navigate("https://www.youtube.com").await
        .expect("navigate failed");

    // Wait for page to fully load
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // 1. Capture full MHTML (HTML + JS + CSS + images)
    println!("[1] Capturing MHTML...");
    let mhtml = page.capture_mhtml().await.expect("mhtml failed");
    std::fs::write("youtube.mhtml", &mhtml).expect("write failed");
    println!("    Saved youtube.mhtml ({} bytes)", mhtml.len());

    // 2. Capture HTML via DOM (scripts intact, no escaping issues)
    println!("[2] Capturing HTML via DOM...");
    let html = page.capture_html().await.expect("html failed");
    std::fs::write("youtube.html", &html).expect("write failed");
    println!("    Saved youtube.html ({} bytes)", html.len());

    // Verify scripts are intact
    let script_count = html.matches("<script").count();
    println!("    <script> tags found: {}", script_count);

    browser.shutdown().await.expect("shutdown failed");
    println!("\nDone. Open youtube.mhtml in Chrome to verify.");
}
