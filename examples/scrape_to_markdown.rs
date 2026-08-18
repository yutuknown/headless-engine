use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;

#[tokio::main]
async fn main() -> Result<()> {
    println!(">>> Launching Headless Engine Tab with Windows Chrome profile...");
    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;

    let url = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
    println!(">>> Navigating to: {}", url);
    let report = tab.navigate(url).await?;

    println!(">>> Page Title: {}", report.page_title);
    println!(">>> Raw HTML Size: {} bytes", report.html_bytes);

    let markdown = tab.extract_markdown(None).unwrap_or_default();
    println!(
        ">>> Markdown Size: {} bytes (~{:.1}% compression)",
        markdown.len(),
        (1.0 - (markdown.len() as f64 / report.html_bytes as f64)) * 100.0
    );

    println!("\n--- MARKDOWN PREVIEW (First 600 chars) ---\n");
    let preview: String = markdown.chars().take(600).collect();
    println!("{}", preview);
    println!("\n-------------------------------------------");

    Ok(())
}
