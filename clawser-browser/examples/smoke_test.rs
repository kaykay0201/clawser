use clawser_browser::Browser;

#[tokio::main]
async fn main() {
    println!("=== Smoke Test ===\n");

    let browser = Browser::builder()
        .headful()
        .profile(7, 777)
        .build().await
        .expect("launch failed");

    let page = browser.navigate("about:blank").await.expect("nav failed");

    let checks = &[
        ("webdriver", "navigator.webdriver.toString()"),
        ("timezone", "Intl.DateTimeFormat().resolvedOptions().timeZone"),
        ("cores", "navigator.hardwareConcurrency.toString()"),
        ("screen", "screen.width+'x'+screen.height"),
        ("UA", "navigator.userAgent"),
    ];
    for (name, code) in checks {
        let val = page.js(code).await.unwrap_or_default();
        println!("{}: {}", name, val);
    }

    // Check if clawser config is loaded
    let cmdline = page.js(
        "(function(){try{return 'check console for --clawser-config'}catch(e){return e.message}})()"
    ).await.unwrap_or_default();
    println!("note: {}", cmdline);

    println!("\nBrowser open. Ctrl+C to exit.");
    loop { tokio::time::sleep(std::time::Duration::from_secs(60)).await; }
}
