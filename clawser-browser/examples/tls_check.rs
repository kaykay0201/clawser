//! Check TLS/HTTP2 fingerprint of our browser vs stock Chrome.

use clawser_browser::Browser;

fn load_dotenv() {
    for path in ["clawser-browser/.env", ".env"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') { continue; }
                if let Some((k, v)) = line.split_once('=') {
                    std::env::set_var(k.trim(), v.trim());
                }
            }
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    load_dotenv();
    println!("=== TLS/HTTP2 Fingerprint Check ===\n");

    let browser = Browser::builder()
        .headful()
        .random()
        .build()
        .await
        .expect("browser launch failed");

    let page = browser.new_page("about:blank").await.expect("page failed");

    // 1. Check TLS fingerprint
    println!("--- tls.peet.ws ---");
    page.goto("https://tls.peet.ws/api/all").await.expect("nav failed");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let body = page.js("document.body.innerText").await.unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
        // TLS
        if let Some(tls) = v.get("tls") {
            println!("JA3 hash:    {}", tls.get("ja3_hash").and_then(|v| v.as_str()).unwrap_or("N/A"));
            println!("JA3:         {}", tls.get("ja3").and_then(|v| v.as_str()).unwrap_or("N/A"));
            println!("JA4:         {}", tls.get("ja4").and_then(|v| v.as_str()).unwrap_or("N/A"));
            println!("TLS version: {}", tls.get("version").and_then(|v| v.as_str()).unwrap_or("N/A"));
        }

        // HTTP2
        println!();
        if let Some(h2) = v.get("http2") {
            println!("HTTP2 fingerprint:   {}", h2.get("akamai_fingerprint").and_then(|v| v.as_str()).unwrap_or("N/A"));
            println!("Pseudo-header order: {}", h2.get("sent_pseudo_header_fields_order").unwrap_or(&serde_json::Value::Null));
            if let Some(settings) = h2.get("sent_settings") {
                println!("SETTINGS:            {}", settings);
            }
        }

        // HTTP version used
        println!();
        if let Some(ip) = v.get("ip") {
            println!("Client IP:   {}", ip.as_str().unwrap_or("N/A"));
        }
        if let Some(http_version) = v.get("http_version") {
            println!("HTTP version: {}", http_version.as_str().unwrap_or("N/A"));
        }

        // User-Agent
        println!();
        if let Some(h1) = v.get("http1") {
            if let Some(headers) = h1.get("headers") {
                if let Some(arr) = headers.as_array() {
                    for h in arr {
                        let name = h.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let val = h.get("value").and_then(|n| n.as_str()).unwrap_or("");
                        if name.to_lowercase() == "user-agent" {
                            println!("User-Agent:  {}", val);
                        }
                    }
                }
            }
        }
    } else {
        println!("Failed to parse response: {}", &body[..body.len().min(500)]);
    }

    // 2. Check known Chrome 134 reference values
    println!("\n--- Expected Chrome 134 stock values ---");
    println!("JA3 hash:    cd49876058a26ccf43e241ccc4e4b5d3  (Chrome 134 Windows)");
    println!("JA4:         t13d1517h2_8daaf6152771_02713d6af8  (Chrome 134 Windows)");
    println!("HTTP2 Akamai: 1:65536;2:0;4:6291456;6:262144|15663105|0|m,a,s,p");

    println!("\nBrowser still open. Press Ctrl+C to exit.");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
