//! Headful YouTube test with antidetect + human simulation.
//!
//! Run: CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example youtube_test

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

    println!("=== YouTube Headful Test (async) ===\n");

    let browser = Browser::builder().headful().config(&config).build().await
        .expect("failed to create browser");

    let page = browser.navigate("https://www.youtube.com").await
        .expect("failed to navigate");

    // Simulate human behavior
    page.human_idle(1500).await.expect("idle failed");
    page.scroll(200).await.expect("scroll failed");

    println!("Title: {}", page.js("document.title").await.unwrap_or_default());
    println!("URL: {}", page.js("window.location.href").await.unwrap_or_default());

    let checks: &[(&str, &str)] = &[
        ("UA", "navigator.userAgent"),
        ("Cores", "navigator.hardwareConcurrency.toString()"),
        ("Memory", "(navigator.deviceMemory||'?').toString()"),
        ("webdriver", "navigator.webdriver.toString()"),
        ("Screen", "screen.width+'x'+screen.height"),
        ("Timezone", "Intl.DateTimeFormat().resolvedOptions().timeZone"),
        ("GPU", "(function(){var c=document.createElement('canvas');var g=c.getContext('webgl');if(!g)return'no';var d=g.getExtension('WEBGL_debug_renderer_info');return d?g.getParameter(d.UNMASKED_RENDERER_WEBGL):'no'})()"),
    ];
    for (name, code) in checks {
        println!("{}: {}", name, page.js(code).await.unwrap_or_default());
    }

    browser.shutdown().await.expect("shutdown failed");
    println!("\nDone.");
}
