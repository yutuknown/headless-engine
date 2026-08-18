use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;

#[tokio::main]
async fn main() -> Result<()> {
    println!("================================================================================");
    println!(">>> AI AGENTIC AUTONOMOUS BROWSING LOOP (PURE RUST)");
    println!("================================================================================\n");

    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;

    // Step 1: Initial Navigation
    println!("[Step 1: Navigate] Agent navigating to Wikipedia Main Page...");
    tab.navigate("https://en.wikipedia.org/wiki/Main_Page")
        .await?;

    // Step 2: Observe
    println!("\n[Step 2: Observe] Extracting Indexed Action Tree for Agent LLM...");
    let obs = tab.observe().expect("Expected observation");
    println!("  * Page Title: {}", obs.title);
    println!(
        "  * Total Interactive Elements Indexed: {}",
        obs.interactive_elements.len()
    );

    println!("\n>>> AGENT ACTION MAP (Sample First 10 Elements):");
    println!("--------------------------------------------------------------------------------");
    for el in obs.interactive_elements.iter().take(10) {
        println!("{}", el.to_agent_string());
    }
    println!("--------------------------------------------------------------------------------");

    // Step 3: Agent Decision & Action
    // Agent chooses to click the search input or a featured article
    if let Some(target_link) = obs
        .interactive_elements
        .iter()
        .find(|e| e.tag == "a" && e.text.contains("article"))
    {
        println!(
            "\n[Step 3: Act] Agent decides to click element [{}] (Text: \"{}\")...",
            target_link.index, target_link.text
        );
        let action_res = tab.act_click(&target_link.index.to_string()).await?;

        if let Some(report) = action_res {
            println!("  -> Navigated to: {}", report.final_url);
            println!("  -> New Page Title: {}", report.page_title);
        }
    }

    // Step 4: Extract LLM Knowledge
    println!("\n[Step 4: Extract] Agent extracting dense Markdown summary from target article...");
    let md = tab.extract_markdown(None).unwrap_or_default();
    println!("  * Extracted Markdown Length: {} bytes", md.len());
    let preview: String = md.chars().take(400).collect();
    println!("\n>>> AGENT KNOWLEDGE PREVIEW:\n{}", preview);

    println!("\n================================================================================");
    println!(">>> AUTONOMOUS AGENT LOOP COMPLETED SEAMLESSLY!");
    println!("================================================================================");

    Ok(())
}
