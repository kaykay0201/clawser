//! Test mouse/keyboard/scroll simulation (async).
//!
//! Run: CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example input_test

use clawser_browser::Browser;

#[tokio::main]
async fn main() {
    let config = std::env::var("CLAWSER_CONFIG").unwrap_or_else(|_| {
        if let Ok(chrome) = std::env::var("CLAWSER_CHROME_PATH") {
            let dir = std::path::Path::new(&chrome).parent().unwrap_or(std::path::Path::new("."));
            let cfg = dir.join("test_profile.json");
            if cfg.exists() { return cfg.to_string_lossy().to_string(); }
        }
        "out/Default/test_profile.json".to_string()
    });

    println!("=== Input Simulation Test (async) ===\n");

    let browser = Browser::builder().headful().config(&config).build().await
        .expect("failed to create browser");
    let page = browser.navigate("https://www.google.com").await
        .expect("navigate failed");

    println!("[1] Human idle (2s)...");
    page.human_idle(2000).await.expect("idle failed");

    println!("[2] Scroll...");
    page.scroll(300).await.expect("scroll failed");

    println!("[3] Click...");
    page.click(600.0, 300.0).await.expect("click failed");

    println!("[4] Type...");
    page.type_text("hello").await.expect("type failed");

    println!("[5] Shutting down...");
    browser.shutdown().await.expect("shutdown failed");

    println!("\n=== PASSED ===");
}
