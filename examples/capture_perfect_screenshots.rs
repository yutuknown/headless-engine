use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("================================================================================");
    println!(">>> CAPTURING HIGH-FIDELITY SCREENSHOTS FOR CHATGPT & GOOGLE");
    println!("================================================================================\n");

    let artifact_dir = Path::new(
        r"C:\Users\abhis\.gemini\antigravity-ide\brain\c08da294-7846-44b1-9403-559e0d23ce0f",
    );
    let evidence_dir = Path::new("evidence");

    // 1. ChatGPT
    println!("[1/3] Capturing ChatGPT (chatgpt.com)...");
    let mut tab1 = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    tab1.navigate("https://chatgpt.com/").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    if let Some(shot) = tab1.screenshot_async().await {
        println!("  -> ChatGPT Screenshot Bytes: {}", shot.png_bytes.len());
        if !shot.png_bytes.is_empty() {
            fs::write(evidence_dir.join("chatgpt_screenshot.png"), &shot.png_bytes)?;
            fs::write(artifact_dir.join("chatgpt_screenshot.png"), &shot.png_bytes)?;
        }
    }

    // 2. Google AI Mode (udm=50)
    println!("\n[2/3] Capturing Google AI Mode (udm=50)...");
    let mut tab2 = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    tab2.navigate("https://www.google.com/search?q=Rust+programming+language+features&udm=50")
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    if let Some(shot) = tab2.screenshot_async().await {
        println!(
            "  -> Google AI Mode Screenshot Bytes: {}",
            shot.png_bytes.len()
        );
        if !shot.png_bytes.is_empty() {
            fs::write(
                evidence_dir.join("google_aimode_screenshot.png"),
                &shot.png_bytes,
            )?;
            fs::write(
                artifact_dir.join("google_aimode_screenshot.png"),
                &shot.png_bytes,
            )?;
        }
    }

    // 3. Google Normal Search
    println!("\n[3/3] Capturing Google Normal Search...");
    let mut tab3 = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    tab3.navigate("https://www.google.com/search?q=headless+browser+pure+rust+engine")
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    if let Some(shot) = tab3.screenshot_async().await {
        println!(
            "  -> Google Normal Search Screenshot Bytes: {}",
            shot.png_bytes.len()
        );
        if !shot.png_bytes.is_empty() {
            fs::write(
                evidence_dir.join("google_normal_search_screenshot.png"),
                &shot.png_bytes,
            )?;
            fs::write(
                artifact_dir.join("google_normal_search_screenshot.png"),
                &shot.png_bytes,
            )?;
        }
    }

    println!("\n>>> Finished capturing screenshots.");
    Ok(())
}
