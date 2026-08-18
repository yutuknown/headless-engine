use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    println!(">>> Testing Pure-Rust Vector SVG & Wireframe Screenshot Rendering...");

    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    tab.navigate("https://news.ycombinator.com/").await?;

    let shot = tab.screenshot().expect("Expected screenshot");
    println!("  * Dimensions: {}x{}", shot.width, shot.height);
    println!("  * Elements Rendered: {}", shot.element_count);
    println!("  * SVG Payload Size: {} bytes", shot.svg.len());

    // Save SVG file
    fs::write("screenshot.svg", &shot.svg)?;
    println!("  * Saved screenshot to 'screenshot.svg'");

    // Print text wireframe preview
    println!("\n>>> ASCII WIREFRAME PREVIEW (FOR AGENT VISION TOKENS):");
    let preview: String = shot
        .layout_wireframe
        .lines()
        .take(18)
        .collect::<Vec<_>>()
        .join("\n");
    println!("{}", preview);

    Ok(())
}
