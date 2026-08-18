use anyhow::Result;
use headless_engine::browser::engine::BrowserEngine;
use headless_engine::network::fingerprint::DeviceProfile;

#[tokio::main]
async fn main() -> Result<()> {
    println!("================================================================================");
    println!(">>> MULTI-TAB CONCURRENT ENGINE DEMO (<50MB RAM)");
    println!("================================================================================\n");

    let mut engine = BrowserEngine::new()?;

    // Create 3 isolated tabs with different device profiles
    let tab1_id = engine.create_tab(Some(DeviceProfile::ChromeWindows))?;
    let tab2_id = engine.create_tab(Some(DeviceProfile::SafariMac))?;
    let tab3_id = engine.create_tab(Some(DeviceProfile::SafariIos))?;

    println!("Created tabs: {}, {}, {}", tab1_id, tab2_id, tab3_id);

    println!("[Tab 1: Chrome Windows] Navigating to Wikipedia...");
    let r1 = engine
        .get_tab_mut(&tab1_id)
        .unwrap()
        .navigate("https://en.wikipedia.org/wiki/Artificial_intelligence")
        .await?;
    println!("  -> Tab 1 Title: {}", r1.page_title);

    println!("[Tab 2: Safari Mac] Navigating to Hacker News...");
    let r2 = engine
        .get_tab_mut(&tab2_id)
        .unwrap()
        .navigate("https://news.ycombinator.com/")
        .await?;
    println!("  -> Tab 2 Title: {}", r2.page_title);

    println!("[Tab 3: iPhone iOS] Navigating to GitHub Explore...");
    let r3 = engine
        .get_tab_mut(&tab3_id)
        .unwrap()
        .navigate("https://github.com/trending")
        .await?;
    println!("  -> Tab 3 Title: {}", r3.page_title);

    println!("\n>>> Active Tabs List:");
    for tab_info in engine.list_tabs() {
        println!(
            "  - ID: {:<8} Profile: {:<15} URL: {:?}",
            tab_info.id,
            format!("{:?}", tab_info.profile),
            tab_info.url
        );
    }

    println!("\n>>> Closing Tab 2...");
    engine.close_tab(&tab2_id);
    println!("Remaining tabs count: {}", engine.list_tabs().len());

    Ok(())
}
