//! Profile rotation with random async browsers.
//!
//! Run: CLAWSER_CHROME_PATH=out/Default/chrome.exe cargo run --manifest-path clawser-browser/Cargo.toml --example rotate_test

use clawser_browser::Browser;

#[tokio::main]
async fn main() {
    println!("=== Profile Rotation Test (async) ===\n");

    for i in 0..3 {
        println!("--- Browser {} (random profile) ---", i + 1);

        let browser = Browser::builder().headful().random().build().await
            .expect("failed to create browser");

        let page = browser.navigate("about:blank").await.expect("navigate failed");

        let cores = page.js("navigator.hardwareConcurrency.toString()").await.unwrap_or_default();
        let screen = page.js("screen.width+'x'+screen.height").await.unwrap_or_default();
        let tz = page.js("Intl.DateTimeFormat().resolvedOptions().timeZone").await.unwrap_or_default();
        let langs = page.js("JSON.stringify(navigator.languages)").await.unwrap_or_default();
        let gl = page.js(
            "(function(){var c=document.createElement('canvas');var g=c.getContext('webgl');if(!g)return'no';var d=g.getExtension('WEBGL_debug_renderer_info');return d?g.getParameter(d.UNMASKED_RENDERER_WEBGL):'no'})()"
        ).await.unwrap_or_default();

        println!("  Cores: {}  Screen: {}", cores, screen);
        println!("  GPU: {}", gl);
        println!("  TZ: {}  Langs: {}", tz, langs);

        browser.shutdown().await.expect("shutdown failed");
        println!();
    }

    println!("=== DONE ===");
}
