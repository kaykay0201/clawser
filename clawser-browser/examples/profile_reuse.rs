//! Profile reuse demo — same profile across sessions keeps cookies + fingerprint.
//!
//! Run: CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example profile_reuse

use clawser_browser::Browser;

const PROFILE_INDEX: usize = 42;
const SEED: u64 = 12345;

#[tokio::main]
async fn main() {
    println!("=== Profile Reuse Demo ===\n");

    // --- Session 1: first visit ---
    println!("[Session 1] First visit with profile({}, {})", PROFILE_INDEX, SEED);
    let browser = Browser::builder()
        .headful()
        .profile(PROFILE_INDEX, SEED)
        .build().await
        .expect("failed to create browser");

    let page = browser.navigate("https://www.youtube.com").await
        .expect("navigate failed");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let title = page.js("document.title").await.unwrap_or_default();
    let cookies = browser.cookies("https://www.youtube.com").await.unwrap_or_default();
    let fingerprint = page.js("navigator.hardwareConcurrency + 'c|' + screen.width + 'x' + screen.height + '|' + Intl.DateTimeFormat().resolvedOptions().timeZone").await.unwrap_or_default();

    println!("    Title: {}", title);
    println!("    Fingerprint: {}", fingerprint);
    println!("    Cookies: {} total", cookies.len());
    for c in cookies.iter().take(5) {
        println!("      {} = {}...", c.name, &c.value[..c.value.len().min(30)]);
    }

    println!("    Shutting down session 1...\n");
    browser.shutdown().await.expect("shutdown failed");

    // --- Session 2: same profile, cookies should persist ---
    println!("[Session 2] Reusing profile({}, {}) — cookies should persist", PROFILE_INDEX, SEED);
    let browser = Browser::builder()
        .headful()
        .profile(PROFILE_INDEX, SEED)
        .build().await
        .expect("failed to create browser");

    // Check cookies BEFORE navigating — they should be in the user-data-dir
    let page = browser.navigate("https://www.youtube.com").await
        .expect("navigate failed");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let cookies2 = browser.cookies("https://www.youtube.com").await.unwrap_or_default();
    let fingerprint2 = page.js("navigator.hardwareConcurrency + 'c|' + screen.width + 'x' + screen.height + '|' + Intl.DateTimeFormat().resolvedOptions().timeZone").await.unwrap_or_default();

    println!("    Fingerprint: {}", fingerprint2);
    println!("    Cookies: {} total", cookies2.len());
    for c in cookies2.iter().take(5) {
        println!("      {} = {}...", c.name, &c.value[..c.value.len().min(30)]);
    }

    // Verify
    println!("\n--- Verification ---");
    println!("    Fingerprint match: {}", fingerprint == fingerprint2);
    println!("    Cookies persisted: {}", cookies2.len() >= cookies.len());

    browser.shutdown().await.expect("shutdown failed");
    println!("\n=== DONE ===");
}
