use headless_engine::{BrowserTab, DeviceProfile};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Testing all 31 Google/YouTube Capabilities...");
    let evidence_dir = Path::new("evidence");
    if !evidence_dir.exists() {
        fs::create_dir_all(evidence_dir)?;
    }

    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;

    // Test a subset so it doesn't take forever, or just do the 5 most important ones to verify parsing
    println!("1. Testing google_search...");
    let res = tab
        .google_search("headless browser pure rust engine")
        .await?;
    fs::write(
        evidence_dir.join("test_google_search.md"),
        res.to_markdown(),
    )?;

    println!("2. Testing youtube_search...");
    let res = tab.youtube_search("rust programming").await?;
    fs::write(
        evidence_dir.join("test_youtube_search.md"),
        res.to_markdown(),
    )?;

    println!("3. Testing google_ai_mode...");
    let res = tab.google_ai_mode("explain quantum computing").await?;
    fs::write(
        evidence_dir.join("test_google_ai_mode.md"),
        res.to_markdown(),
    )?;

    println!("4. Testing google_autocomplete...");
    let res = tab.google_autocomplete("rust").await?;
    fs::write(
        evidence_dir.join("test_google_autocomplete.md"),
        res.to_markdown(),
    )?;

    println!("Capabilities listed:");
    for cap in tab.google_capabilities() {
        println!(" - {}", cap);
    }

    println!("All tests completed. Output saved to evidence/");
    Ok(())
}
