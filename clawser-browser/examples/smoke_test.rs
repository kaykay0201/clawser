use clawser_browser::Browser;

#[tokio::main]
async fn main() {
    println!("=== Smoke Test ===\n");

    let browser = Browser::builder()
        .headful()
        .profile(7, 777)
        .build()
        .await
        .expect("launch failed");

    let page = browser.new_page("about:blank").await.expect("page failed");

    let checks = &[
        ("webdriver", "navigator.webdriver.toString()"),
        ("timezone", "Intl.DateTimeFormat().resolvedOptions().timeZone"),
        ("cores", "navigator.hardwareConcurrency.toString()"),
        ("screen", "screen.width+'x'+screen.height"),
        ("UA", "navigator.userAgent"),
    ];
    for (name, expr) in checks {
        let val = page.js(expr).await.unwrap_or_default();
        println!("{name}: {val}");
    }

    println!("\nBrowser open. Ctrl+C to exit.");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
