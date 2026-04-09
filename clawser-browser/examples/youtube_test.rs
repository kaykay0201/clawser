use clawser_browser::Browser;

#[tokio::main]
async fn main() {
    println!("=== YouTube Test ===\n");

    let browser = Browser::builder()
        .headful()
        .profile(7, 777)
        .build()
        .await
        .expect("launch failed");

    let page = browser.new_page("https://www.youtube.com").await.expect("nav failed");
    page.wait(3000).await;

    let title = page.js("document.title").await.unwrap_or_default();
    let url = page.url().await.unwrap_or_default();
    println!("Title: {title}");
    println!("URL: {url}");

    println!("\nBrowser open. Ctrl+C to exit.");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
