# just-fetch

Async browser automation crate powered by Chrome CDP with built-in device profile rotation. Native tokio support.

## Features

- **Headful & headless** — both modes supported via `chrome.exe`
- **100 device profiles** — realistic hardware configs (Steam Survey data), static in binary
- **Profile rotation** — random or deterministic (same seed = same fingerprint)
- **Cookie persistence** — same profile across sessions keeps cookies
- **Human input simulation** — mouse (bezier curves), keyboard (variable timing), scroll (momentum)
- **Page capture** — MHTML (full page) or HTML (DOM-level, no escaping issues)
- **Screenshot** — PNG capture via CDP
- **Async native** — all methods are `async`, works with `tokio::spawn`, `join!`, `select!`

## Quick Start

```rust
use just_fetch::Browser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::builder()
        .headful()
        .random()           // random device profile
        .build().await?;

    let page = browser.navigate("https://example.com").await?;

    // Human behavior simulation
    page.human_idle(2000).await?;
    page.click(400.0, 300.0).await?;
    page.type_text("search query").await?;

    // JS evaluation
    let title = page.js("document.title").await?;
    println!("Title: {}", title);

    // Page capture
    let html = page.capture_html().await?;
    let mhtml = page.capture_mhtml().await?;

    // Screenshot
    let png = browser.screenshot().await?;

    // Cookies persist across sessions with same profile
    let cookies = browser.cookies("https://example.com").await?;

    browser.shutdown().await?;
    Ok(())
}
```

## Profile Rotation

```rust
// Random profile each time (100 hardware × unlimited seeds)
Browser::builder().random().build().await?;

// Deterministic: same (index, seed) = same fingerprint always
Browser::builder().profile(42, 12345).build().await?;

// Custom config file
Browser::builder().config("path/to/profile.json").build().await?;
```

## Requirements

- Chrome/Chromium executable in PATH or set `CLAWSER_CHROME_PATH` env var
- Windows/Linux/macOS

## License

MIT
