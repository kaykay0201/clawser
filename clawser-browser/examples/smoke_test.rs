//! Smoke test: headless + headful antidetect via chrome.exe + CDP.
//!
//! Run: CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example smoke_test

use clawser_browser::Browser;
use std::time::Instant;

async fn test_antidetect(page: &clawser_browser::Page<'_>, label: &str) {
    println!("\n  --- {} antidetect checks ---", label);
    let checks: &[(&str, &str)] = &[
        ("UA", "navigator.userAgent"),
        ("Platform", "navigator.platform"),
        ("Languages", "JSON.stringify(navigator.languages)"),
        ("Cores", "navigator.hardwareConcurrency.toString()"),
        ("webdriver", "navigator.webdriver.toString()"),
        ("Screen", "screen.width + 'x' + screen.height"),
        ("Timezone", "Intl.DateTimeFormat().resolvedOptions().timeZone"),
        ("chrome", "typeof window.chrome"),
        ("cdc_", "(function(){for(var k in window){if(/^cdc_/.test(k))return 'YES'}return 'NO'})()"),
    ];
    for (name, code) in checks {
        match page.js(code).await {
            Ok(val) => println!("    {}: {}", name, val),
            Err(e) => println!("    {}: ERROR - {}", name, e),
        }
    }
}

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

    println!("=== just-fetch smoke test (async) ===\n");

    // HEADLESS
    println!("[1] Creating HEADLESS browser...");
    let start = Instant::now();
    let browser = Browser::builder().headless().config(&config).build().await
        .expect("Failed to create headless browser");
    println!("    OK in {:?}", start.elapsed());

    let page = browser.navigate("about:blank").await.expect("navigate failed");
    let result = page.js("1 + 1").await.expect("js failed");
    assert!(result.contains("2"));
    test_antidetect(&page, "HEADLESS").await;
    browser.shutdown().await.expect("shutdown failed");

    // HEADFUL
    println!("\n[2] Creating HEADFUL browser...");
    let start = Instant::now();
    let browser = Browser::builder().headful().config(&config).build().await
        .expect("Failed to create headful browser");
    println!("    OK in {:?}", start.elapsed());

    let page = browser.navigate("about:blank").await.expect("navigate failed");
    test_antidetect(&page, "HEADFUL").await;
    browser.shutdown().await.expect("shutdown failed");

    println!("\n=== ALL TESTS PASSED ===");
}
