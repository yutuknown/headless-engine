use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    println!("================================================================================");
    println!(">>> REAL RASTER PNG SCREENSHOT TEST (PURE RUST RESVG + TINY-SKIA)");
    println!("================================================================================\n");

    let url = "https://www.youtube.com/results?search_query=more+suhagan";
    println!("[1] Navigating to target URL: {}", url);

    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    let report = tab.navigate(url).await?;

    println!("  * HTTP Status: {}", report.status);
    println!("  * Final URL: {}", report.final_url);
    println!("  * Page Title: {}", report.page_title);
    println!("  * HTML Size: {} bytes", report.html_bytes);

    // 2. Extract multi-modal video results
    let results = tab
        .extract_search_results()
        .expect("Expected search results");
    println!(
        "\n[2] Extracted Video Results Found: {}",
        results.video_results.len()
    );
    for (i, v) in results.video_results.iter().take(5).enumerate() {
        println!("  [{}] Title: {}", i + 1, v.title);
        println!("      Channel: {} | Duration: {}", v.channel, v.duration);
    }

    // 3. Capture REAL PNG Screenshot
    println!("\n[3] Capturing Real Binary PNG Screenshot with live image assets...");
    let shot = tab.screenshot_async().await.expect("Expected screenshot");
    println!("  * Dimensions: {}x{}", shot.width, shot.height);
    println!("  * Rendered Visual Elements: {}", shot.element_count);
    println!("  * Real PNG File Size: {} bytes", shot.png_bytes.len());
    println!("  * Base64 Data URL Prefix: {}...", &shot.png_base64[..50]);

    // Save actual PNG binary file
    fs::write("youtube_screenshot.png", &shot.png_bytes)?;
    println!("  -> Successfully wrote real binary image: 'youtube_screenshot.png'");

    // Save SVG file
    fs::write("youtube_screenshot.svg", &shot.svg)?;
    println!("  -> Successfully wrote vector SVG file: 'youtube_screenshot.svg'");

    println!("\n================================================================================");
    println!(">>> REAL PNG SCREENSHOT GENERATED WITHOUT CHROMIUM (<30MB RAM)!");
    println!("================================================================================");

    Ok(())
}
