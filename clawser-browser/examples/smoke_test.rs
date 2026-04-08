//! Smoke test for clawser-browser.
//!
//! Tests: init → navigate → js eval → shutdown.
//! Run with: CLAWSER_BROWSER_PATH=out/Default/clawser_browser.exe cargo run --example smoke_test

use clawser_browser::Browser;
use std::time::Instant;

fn main() {
    println!("=== clawser-browser smoke test ===\n");

    // 1. Create browser with random profile
    println!("[1/5] Creating browser...");
    let start = Instant::now();
    let (browser, seed) = Browser::new().expect("Failed to create browser");
    println!("  OK in {:?}", start.elapsed());
    println!("  Seed: hw={} canvas={} webgl={} audio={} rects={}",
        seed.hw_seed, seed.canvas_seed, seed.webgl_seed,
        seed.audio_seed, seed.client_rects_seed);

    // 2. Navigate to about:blank (safe, no network needed)
    println!("\n[2/5] Navigating to about:blank...");
    let start = Instant::now();
    let page = browser.navigate("about:blank").expect("Failed to navigate");
    println!("  OK in {:?}", start.elapsed());

    // 3. Execute JS
    println!("\n[3/5] Executing JS: 1 + 1...");
    let start = Instant::now();
    let result = page.js("1 + 1").expect("Failed to execute JS");
    println!("  Result: {}", result);
    println!("  OK in {:?}", start.elapsed());
    assert!(result.contains("2"), "Expected '2', got '{}'", result);

    // 4. Check antidetect spoofing
    println!("\n[4/5] Checking antidetect...");
    let platform = page.js("navigator.platform").expect("JS failed");
    println!("  navigator.platform = {}", platform);

    let webdriver = page.js("navigator.webdriver").expect("JS failed");
    println!("  navigator.webdriver = {}", webdriver);

    let chrome_exists = page.js("typeof window.chrome").expect("JS failed");
    println!("  typeof window.chrome = {}", chrome_exists);

    let pointer = page.js("matchMedia('(pointer: fine)').matches").expect("JS failed");
    println!("  (pointer: fine) = {}", pointer);

    // 5. Shutdown
    println!("\n[5/5] Shutting down...");
    let start = Instant::now();
    browser.shutdown().expect("Failed to shutdown");
    println!("  OK in {:?}", start.elapsed());

    println!("\n=== ALL TESTS PASSED ===");
}
