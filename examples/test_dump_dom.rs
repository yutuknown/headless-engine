use anyhow::Result;
use headless_engine::dom::screenshot::RealBrowserScreenshot;

#[tokio::main]
async fn main() -> Result<()> {
    println!(">>> Testing RealBrowserScreenshot::dump_rendered_dom...");
    let url = "https://www.google.com/search?q=Rust+programming+language";
    println!("  -> Target URL: {}", url);
    let start = std::time::Instant::now();
    let res = RealBrowserScreenshot::dump_rendered_dom(url).await;
    println!("  -> Duration: {:?}", start.elapsed());
    if let Some(html) = res {
        println!("  -> Got HTML! Bytes: {}", html.len());
        println!("  -> Contains <h3>: {}", html.contains("<h3"));
        println!("  -> Contains id=\"search\": {}", html.contains("id=\"search\""));
        println!("  -> Contains class=\"g\": {}", html.contains("class=\"g\""));
    } else {
        println!("  -> dump_rendered_dom returned None!");
    }
    Ok(())
}
