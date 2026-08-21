use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;

#[tokio::main]
async fn main() -> Result<()> {
    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    
    let urls = [
        "https://www.google.com/search?q=headless+browser+rust+engine+for+ai+agents&gbv=1",
        "https://www.google.com/search?q=headless+browser+rust+engine+for+ai+agents&udm=14",
        "https://www.google.com/search?q=Rust+programming+language&udm=14",
        "https://html.duckduckgo.com/html/?q=headless+browser+rust+engine+for+ai+agents",
    ];

    for url in urls {
        println!("\n========================================");
        println!("Testing URL: {}", url);
        let nav = tab.navigate(url).await?;
        println!("Status: {}", nav.status);
        println!("Title:  {}", nav.page_title);
        println!("Bytes:  {}", nav.html_bytes);
        let md = tab.extract_markdown(None).unwrap_or_default();
        println!("MD Len: {} bytes", md.len());
        println!("Preview:\n{}", md.chars().take(300).collect::<String>());
    }

    Ok(())
}
