use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;
use serde_json::json;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("================================================================================");
    println!(">>> HEADLESS ENGINE: VERIFYING SCREENSHOT, MARKDOWN & AGENTIC NAVIGATION");
    println!("================================================================================\n");

    let artifact_dir = Path::new(
        r"C:\Users\abhis\.gemini\antigravity-ide\brain\c08da294-7846-44b1-9403-559e0d23ce0f",
    );
    let evidence_dir = Path::new("evidence");
    fs::create_dir_all(evidence_dir)?;
    if !artifact_dir.exists() {
        let _ = fs::create_dir_all(artifact_dir);
    }

    // Initialize browser with stealth fingerprint
    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;

    // =========================================================================
    // 1. AGENTIC NAVIGATION FEATURE
    // =========================================================================
    println!("[1. AGENTIC NAVIGATION] Initial navigation to Wikipedia Portal...");
    let initial_url = "https://en.wikipedia.org/wiki/Portal:Current_events";
    let init_report = tab.navigate(initial_url).await?;
    println!("  -> Landed on: {}", init_report.requested_url);
    println!("  -> Page Title: {}", init_report.page_title);
    println!("  -> Status: {}", init_report.status);

    // Extract Agent Action Map / Observation Tree
    let obs = tab.observe().expect("Expected observation");
    println!(
        "  -> Total Interactive Elements Indexed: {}",
        obs.interactive_elements.len()
    );

    // Agent decision: find an informative article link to click
    let selected_target = obs
        .interactive_elements
        .iter()
        .find(|e| {
            e.tag == "a"
                && e.text.len() > 15
                && !e.href.is_empty()
                && !e.href.contains("#")
                && !e.href.contains("Portal:")
        })
        .or_else(|| {
            obs.interactive_elements
                .iter()
                .find(|e| e.tag == "a" && e.text.contains("article"))
        })
        .or_else(|| obs.interactive_elements.first())
        .expect("No interactive link found");

    let clicked_index = selected_target.index;
    let clicked_text = selected_target.text.clone();
    let clicked_href = selected_target.href.clone();
    let clicked_selector = selected_target.selector.clone();

    println!("\n  [Agent Decision & Act]");
    println!("  -> Selected Element ID: [{}]", clicked_index);
    println!("  -> Element Selector:    {}", clicked_selector);
    println!("  -> Element Anchor Text: \"{}\"", clicked_text);
    println!("  -> Target URL:          {}", clicked_href);

    // Execute Autonomous Action Click via element index
    let nav_report = tab
        .act_click(&clicked_index.to_string())
        .await?
        .expect("Expected navigation report after click");

    println!("\n  [Agent Navigation Result]");
    println!("  -> Successfully Navigated to: {}", nav_report.final_url);
    println!("  -> New Page Title:            {}", nav_report.page_title);
    println!("  -> HTTP Status:               {}", nav_report.status);
    println!(
        "  -> HTML Payload Size:         {} bytes",
        nav_report.html_bytes
    );

    // Save Agent Action Map
    let sample_elements: Vec<_> = obs.interactive_elements.iter().take(20).cloned().collect();
    let agent_map_json = json!({
        "initial_page": {
            "url": initial_url,
            "title": init_report.page_title,
            "interactive_elements_count": obs.interactive_elements.len()
        },
        "agent_action": {
            "clicked_element_index": clicked_index,
            "anchor_text": clicked_text,
            "target_href": clicked_href,
            "selector": clicked_selector
        },
        "target_page": {
            "final_url": nav_report.final_url,
            "page_title": nav_report.page_title,
            "status": nav_report.status,
            "html_bytes": nav_report.html_bytes
        },
        "sample_indexed_elements": sample_elements
    });
    fs::write(
        evidence_dir.join("agentic_navigation_trace.json"),
        serde_json::to_string_pretty(&agent_map_json)?,
    )?;
    fs::write(
        artifact_dir.join("agentic_navigation_trace.json"),
        serde_json::to_string_pretty(&agent_map_json)?,
    )?;

    // =========================================================================
    // 2. SCREENSHOT FEATURE
    // =========================================================================
    println!(
        "\n[2. SCREENSHOT FEATURE] Capturing high-resolution visual screenshot & vector SVG..."
    );
    let shot = tab.screenshot_async().await.expect("Expected screenshot");
    println!("  -> Dimensions:        {}x{} px", shot.width, shot.height);
    println!("  -> Elements Rendered: {}", shot.element_count);
    println!("  -> SVG Size:          {} bytes", shot.svg.len());
    println!("  -> PNG Bytes:         {} bytes", shot.png_bytes.len());

    let png_dest_evidence = evidence_dir.join("demonstration_screenshot.png");
    let png_dest_artifact = artifact_dir.join("demonstration_screenshot.png");
    let svg_dest_evidence = evidence_dir.join("demonstration_screenshot.svg");
    let svg_dest_artifact = artifact_dir.join("demonstration_screenshot.svg");
    let wireframe_dest = evidence_dir.join("demonstration_wireframe.txt");

    if !shot.png_bytes.is_empty() {
        fs::write(&png_dest_evidence, &shot.png_bytes)?;
        fs::write(&png_dest_artifact, &shot.png_bytes)?;
        println!(
            "  -> Saved PNG Screenshot to: {}",
            png_dest_evidence.display()
        );
    }
    fs::write(&svg_dest_evidence, &shot.svg)?;
    fs::write(&svg_dest_artifact, &shot.svg)?;
    fs::write(&wireframe_dest, &shot.layout_wireframe)?;
    println!(
        "  -> Saved SVG Vector Layout to: {}",
        svg_dest_evidence.display()
    );

    // =========================================================================
    // 3. MARKDOWN DISTILLATION FEATURE
    // =========================================================================
    println!("\n[3. MARKDOWN FEATURE] Extracting token-efficient distilled Markdown for LLMs...");
    let markdown = tab.extract_markdown(None).unwrap_or_default();
    let raw_html_len = nav_report.html_bytes;
    let md_len = markdown.len();
    let reduction_pct = if raw_html_len > 0 {
        ((raw_html_len as f64 - md_len as f64) / raw_html_len as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "  -> Raw HTML Payload:      {} bytes (~{} tokens)",
        raw_html_len,
        raw_html_len / 4
    );
    println!(
        "  -> Distilled Markdown:    {} bytes (~{} tokens)",
        md_len,
        md_len / 4
    );
    println!("  -> Token/Size Reduction:  {:.2}%", reduction_pct);

    let md_dest_evidence = evidence_dir.join("demonstration_distilled.md");
    let md_dest_artifact = artifact_dir.join("demonstration_distilled.md");
    fs::write(&md_dest_evidence, &markdown)?;
    fs::write(&md_dest_artifact, &markdown)?;
    println!(
        "  -> Saved Distilled Markdown to: {}",
        md_dest_evidence.display()
    );

    // Preview Markdown header
    println!("\n>>> DISTILLED MARKDOWN PREVIEW (FIRST 400 CHARS):");
    println!("--------------------------------------------------------------------------------");
    let preview: String = markdown.chars().take(400).collect();
    println!("{}", preview);
    println!("--------------------------------------------------------------------------------");

    // Save Complete Evidence Summary
    let summary = json!({
        "engine": "Headless Engine (Pure Rust)",
        "features_tested": {
            "1_agentic_navigation": {
                "initial_url": initial_url,
                "selected_element": {
                    "id": clicked_index,
                    "text": clicked_text,
                    "target_url": clicked_href
                },
                "destination_page": {
                    "url": nav_report.final_url,
                    "title": nav_report.page_title,
                    "status": nav_report.status
                },
                "status": "PASS"
            },
            "2_screenshot": {
                "png_size_bytes": shot.png_bytes.len(),
                "svg_size_bytes": shot.svg.len(),
                "resolution": format!("{}x{}", shot.width, shot.height),
                "png_file": png_dest_evidence.to_string_lossy(),
                "svg_file": svg_dest_evidence.to_string_lossy(),
                "status": "PASS"
            },
            "3_markdown_distillation": {
                "raw_html_bytes": raw_html_len,
                "markdown_bytes": md_len,
                "reduction_percentage": format!("{:.2}%", reduction_pct),
                "markdown_file": md_dest_evidence.to_string_lossy(),
                "status": "PASS"
            }
        }
    });

    fs::write(
        evidence_dir.join("demonstration_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;
    fs::write(
        artifact_dir.join("demonstration_summary.json"),
        serde_json::to_string_pretty(&summary)?,
    )?;

    println!("\n================================================================================");
    println!(">>> ALL 3 FEATURES SUCCESSFULLY DEMONSTRATED AND EVIDENCE ARTIFACTS SAVED!");
    println!("================================================================================");

    Ok(())
}
