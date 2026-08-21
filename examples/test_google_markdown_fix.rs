use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!(">>> Testing Google Search Distilled Markdown Extraction...");

    let artifact_dir = Path::new(
        r"C:\Users\abhis\.gemini\antigravity-ide\brain\c08da294-7846-44b1-9403-559e0d23ce0f",
    );
    let evidence_dir = Path::new("evidence");

    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    let url = "https://www.google.com/search?q=headless+browser+rust+engine+for+ai+agents";
    println!("  -> Navigating to: {}", url);
    let nav = tab.navigate(url).await?;

    println!("  -> Status:       {}", nav.status);
    println!("  -> Final URL:    {}", nav.final_url);
    println!("  -> Page Title:   {}", nav.page_title);
    println!("  -> HTML Payload: {} bytes", nav.html_bytes);

    let md = tab.extract_markdown(None).unwrap_or_default();
    println!("  -> Extracted Markdown Size: {} bytes", md.len());

    fs::write(evidence_dir.join("google_normal_search_distilled.md"), &md)?;
    fs::write(artifact_dir.join("google_normal_search_distilled.md"), &md)?;

    println!("\n>>> DISTILLED GOOGLE SEARCH MARKDOWN:\n");
    println!("{}", md);

    Ok(())
}
