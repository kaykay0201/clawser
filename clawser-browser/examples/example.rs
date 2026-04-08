//! Example: Launch antidetect browser, navigate, eval JS, fetch, cookies.
//!
//! Run with:
//!   CLAWSER_BROWSER_PATH=out/Default/clawser_browser.exe \
//!     cargo run --manifest-path clawser-browser/Cargo.toml --example example

use clawser_browser::Browser;

fn main() {
    // --- Create browser with random antidetect profile ---
    let (browser, seed) = Browser::new().expect("failed to create browser");
    println!("Browser launched, hw_seed={}", seed.hw_seed);

    // --- Navigate to a page ---
    let page = browser.navigate("https://httpbin.org/get")
        .expect("failed to navigate");
    println!("Navigated to httpbin.org");

    // --- Execute JS in the page ---
    let ua = page.js("navigator.userAgent").expect("js failed");
    println!("User-Agent: {}", ua);

    let platform = page.js("navigator.platform").expect("js failed");
    println!("Platform: {}", platform);

    let webdriver = page.js("navigator.webdriver").expect("js failed");
    println!("webdriver: {}", webdriver);

    let chrome = page.js("typeof window.chrome").expect("js failed");
    println!("window.chrome: {}", chrome);

    let cores = page.js("navigator.hardwareConcurrency").expect("js failed");
    println!("Cores: {}", cores);

    let memory = page.js("navigator.deviceMemory").expect("js failed");
    println!("Device memory: {}GB", memory);

    let screen = page.js(
        "screen.width + 'x' + screen.height"
    ).expect("js failed");
    println!("Screen: {}", screen);

    // --- HTTP fetch through the browser's C++ network stack ---
    // Same TLS/H2 fingerprint, same cookies as the page.
    let resp = browser.fetch("GET", "https://httpbin.org/headers")
        .header("X-Custom", "clawser-test")
        .timeout_ms(10_000)
        .send()
        .expect("fetch failed");
    println!("\nFetch status: {}", resp.status);
    println!("Fetch body (first 200 chars): {}",
        String::from_utf8_lossy(&resp.body[..resp.body.len().min(200)]));

    // --- Get cookies ---
    let cookies = browser.cookies("https://httpbin.org")
        .expect("cookies failed");
    println!("\nCookies for httpbin.org: {:?}", cookies);

    // --- Open a WebSocket (C++ Mojo, zero JS injection) ---
    // Uncomment to test with a real WS server:
    // let ws = browser.websocket("wss://echo.websocket.events")
    //     .expect("ws failed");
    // ws.send("hello from clawser").expect("ws send failed");
    // let msg = ws.recv(5000).expect("ws recv failed");
    // println!("WS echo: {}", msg);
    // ws.close().expect("ws close failed");

    // --- Shutdown ---
    browser.shutdown().expect("shutdown failed");
    println!("\nDone.");
}
