use anyhow::Result;
use headless_engine::{BrowserTab, DeviceProfile};
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("================================================================================");
    println!(">>> TESTING GOOGLE AS DEFAULT SEARCH ENGINE (PURE RUST)");
    println!("================================================================================\n");

    let artifact_dir = Path::new(r"C:\Users\abhis\.gemini\antigravity-ide\brain\c08da294-7846-44b1-9403-559e0d23ce0f");
    let evidence_dir = Path::new("evidence");
    fs::create_dir_all(evidence_dir)?;

    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;

    // 1. Default Google Search
    let query = "headless browser pure rust engine for AI agents";
    println!("[1/2] Executing default search: tab.search(\"{}\")...", query);
    let nav = tab.search(query).await?;
    println!("  -> Final URL:     {}", nav.final_url);
    println!("  -> Page Title:    {}", nav.page_title);
    println!("  -> Status:        {}", nav.status);
    println!("  -> HTML Payload:  {} bytes", nav.html_bytes);

    let md = tab.extract_markdown(None).unwrap_or_default();
    println!("  -> Distilled Markdown Size: {} bytes", md.len());

    fs::write(evidence_dir.join("google_normal_search_distilled.md"), &md)?;
    fs::write(artifact_dir.join("google_normal_search_distilled.md"), &md)?;

    println!("\n>>> DISTILLED GOOGLE SEARCH MARKDOWN:\n--------------------------------------------------------------------------------");
    println!("{}", md);
    println!("--------------------------------------------------------------------------------");

    // 2. Google AI Mode Search
    let ai_query = "Rust programming language concurrency patterns";
    println!("\n[2/2] Executing Google AI Mode search: tab.search_google(\"{}\", Some(\"ai\"))...", ai_query);
    let ai_nav = tab.search_google(ai_query, Some("ai")).await?;
    println!("  -> Final URL:     {}", ai_nav.final_url);
    println!("  -> Page Title:    {}", ai_nav.page_title);
    println!("  -> HTML Payload:  {} bytes", ai_nav.html_bytes);

    let ai_md = tab.extract_markdown(None).unwrap_or_default();
    println!("  -> Distilled Markdown Size: {} bytes", ai_md.len());

    fs::write(evidence_dir.join("google_aimode_distilled.md"), &ai_md)?;
    fs::write(artifact_dir.join("google_aimode_distilled.md"), &ai_md)?;

    println!("\n>>> DISTILLED GOOGLE AI MODE MARKDOWN:\n--------------------------------------------------------------------------------");
    println!("{}", ai_md);
    println!("--------------------------------------------------------------------------------");

    println!("\n================================================================================");
    println!(">>> GOOGLE DEFAULT SEARCH ENGINE VERIFICATION COMPLETE!");
    println!("================================================================================");

    Ok(())
}
