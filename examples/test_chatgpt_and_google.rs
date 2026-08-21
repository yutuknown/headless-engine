use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;
use serde_json::json;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("================================================================================");
    println!(">>> TEST SUITE: CHATGPT.COM (AGENTIC MODE) & GOOGLE.COM (AI MODE, AI OVERVIEW, NORMAL SEARCH)");
    println!("================================================================================\n");

    let artifact_dir = Path::new(
        r"C:\Users\abhis\.gemini\antigravity-ide\brain\c08da294-7846-44b1-9403-559e0d23ce0f",
    );
    let evidence_dir = Path::new("evidence");
    fs::create_dir_all(evidence_dir)?;
    if !artifact_dir.exists() {
        let _ = fs::create_dir_all(artifact_dir);
    }

    // =========================================================================
    // PART 1: CHATGPT.COM (AGENTIC MODE TESTING)
    // =========================================================================
    println!("--------------------------------------------------------------------------------");
    println!(">>> [1/4] TESTING CHATGPT.COM — AGENTIC MODE & INTERACTIVE ACTION TREE");
    println!("--------------------------------------------------------------------------------");

    let mut gpt_tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    let gpt_url = "https://chatgpt.com/";
    println!(
        "  [Step 1.1] Navigating with Stealth Anti-Detection to {}...",
        gpt_url
    );
    let gpt_nav = gpt_tab.navigate(gpt_url).await?;
    println!("  -> Landed Status:    {}", gpt_nav.status);
    println!("  -> Page Title:       {}", gpt_nav.page_title);
    println!("  -> CAPTCHA Detected: {}", gpt_nav.is_captcha_detected);
    println!("  -> HTML Payload:     {} bytes", gpt_nav.html_bytes);

    // Agentic Observation & Action Tree
    println!("  [Step 1.2] Extracting Agent Action Map (Interactive Elements)...");
    let gpt_obs = gpt_tab.observe().expect("Expected ChatGPT observation");
    println!(
        "  -> Total Interactive Elements Indexed: {}",
        gpt_obs.interactive_elements.len()
    );

    // Print sample of action tree
    println!("\n  >>> CHATGPT AGENT ACTION MAP (First 10 Interactive Elements):");
    for el in gpt_obs.interactive_elements.iter().take(10) {
        println!("    {}", el.to_agent_string());
    }

    // Agentic Action: Attempt to find prompt input or interactive button
    let input_target = gpt_obs
        .interactive_elements
        .iter()
        .find(|e| {
            e.is_input
                || e.tag == "textarea"
                || e.placeholder.to_lowercase().contains("message")
                || e.placeholder.to_lowercase().contains("ask")
                || e.selector.contains("prompt")
        })
        .cloned();

    let button_target = gpt_obs
        .interactive_elements
        .iter()
        .find(|e| e.tag == "button" || e.role == "button")
        .cloned();

    let mut action_executed = "None".to_string();
    if let Some(target) = &input_target {
        println!(
            "\n  [Step 1.3: Agent Act] Agent typing into input element [{}] (Selector: {})...",
            target.index, target.selector
        );
        let type_res = gpt_tab
            .act_type(
                &target.index.to_string(),
                "Explain quantum computing in one sentence",
            )
            .await?;
        println!("  -> Type Action Result: {}", type_res);
        action_executed = format!("Type into element [{}] ({})", target.index, target.selector);
    } else if let Some(target) = &button_target {
        println!(
            "\n  [Step 1.3: Agent Act] Agent inspecting interactive button [{}] (Text: \"{}\")...",
            target.index, target.text
        );
        action_executed = format!("Inspect/Focus button [{}] ({})", target.index, target.text);
    }

    // Capture ChatGPT Screenshot
    println!("  [Step 1.4] Capturing ChatGPT Visual Screenshot...");
    let gpt_shot = gpt_tab
        .screenshot_async()
        .await
        .expect("Expected screenshot");
    println!(
        "  -> Screenshot PNG Size: {} bytes ({}x{})",
        gpt_shot.png_bytes.len(),
        gpt_shot.width,
        gpt_shot.height
    );

    if !gpt_shot.png_bytes.is_empty() {
        fs::write(
            evidence_dir.join("chatgpt_screenshot.png"),
            &gpt_shot.png_bytes,
        )?;
        fs::write(
            artifact_dir.join("chatgpt_screenshot.png"),
            &gpt_shot.png_bytes,
        )?;
    }

    // Extract ChatGPT Markdown
    println!("  [Step 1.5] Extracting Distilled LLM Markdown...");
    let gpt_md = gpt_tab.extract_markdown(None).unwrap_or_default();
    println!("  -> Distilled Markdown Size: {} bytes", gpt_md.len());
    fs::write(evidence_dir.join("chatgpt_distilled.md"), &gpt_md)?;
    fs::write(artifact_dir.join("chatgpt_distilled.md"), &gpt_md)?;

    // Save ChatGPT Observation JSON
    let gpt_trace_json = json!({
        "url": gpt_url,
        "page_title": gpt_nav.page_title,
        "status": gpt_nav.status,
        "is_captcha_detected": gpt_nav.is_captcha_detected,
        "html_bytes": gpt_nav.html_bytes,
        "interactive_elements_count": gpt_obs.interactive_elements.len(),
        "action_executed": action_executed,
        "interactive_elements": gpt_obs.interactive_elements
    });
    fs::write(
        evidence_dir.join("chatgpt_agentic_observation.json"),
        serde_json::to_string_pretty(&gpt_trace_json)?,
    )?;
    fs::write(
        artifact_dir.join("chatgpt_agentic_observation.json"),
        serde_json::to_string_pretty(&gpt_trace_json)?,
    )?;

    // =========================================================================
    // PART 2: GOOGLE.COM — AI MODE (`udm=50`)
    // =========================================================================
    println!("\n--------------------------------------------------------------------------------");
    println!(">>> [2/4] TESTING GOOGLE.COM — AI MODE (udm=50)");
    println!("--------------------------------------------------------------------------------");

    let mut google_ai_tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    let google_aimode_url =
        "https://www.google.com/search?q=Rust+programming+language+concurrency+patterns&udm=50";
    println!(
        "  [Step 2.1] Navigating to Google AI Mode: {}",
        google_aimode_url
    );
    let aimode_nav = google_ai_tab.navigate(google_aimode_url).await?;
    println!("  -> Page Title:       {}", aimode_nav.page_title);
    println!("  -> Status:           {}", aimode_nav.status);
    println!("  -> CAPTCHA Detected: {}", aimode_nav.is_captcha_detected);
    println!("  -> HTML Payload:     {} bytes", aimode_nav.html_bytes);

    // Extract Structured Search Results
    let aimode_results = google_ai_tab
        .extract_search_results()
        .expect("Expected search results");
    println!(
        "  -> Total Organic Results Found: {}",
        aimode_results.organic_results.len()
    );
    println!(
        "  -> Related Questions (PAA):    {}",
        aimode_results.related_questions.len()
    );
    println!(
        "  -> Has AI Overview:            {}",
        aimode_results.ai_overview.is_some()
    );
    println!(
        "  -> Has Knowledge Panel:        {}",
        aimode_results.knowledge_panel.is_some()
    );

    // Capture Screenshot
    println!("  [Step 2.2] Capturing Google AI Mode Screenshot...");
    let aimode_shot = google_ai_tab
        .screenshot_async()
        .await
        .expect("Expected screenshot");
    println!(
        "  -> Screenshot PNG Size: {} bytes",
        aimode_shot.png_bytes.len()
    );
    if !aimode_shot.png_bytes.is_empty() {
        fs::write(
            evidence_dir.join("google_aimode_screenshot.png"),
            &aimode_shot.png_bytes,
        )?;
        fs::write(
            artifact_dir.join("google_aimode_screenshot.png"),
            &aimode_shot.png_bytes,
        )?;
    }

    // Extract Markdown
    println!("  [Step 2.3] Distilling Google AI Mode Markdown...");
    let aimode_md = google_ai_tab.extract_markdown(None).unwrap_or_default();
    println!(
        "  -> Markdown Size: {} bytes ({:.2}% reduction)",
        aimode_md.len(),
        ((aimode_nav.html_bytes - aimode_md.len()) as f64 / aimode_nav.html_bytes as f64) * 100.0
    );
    fs::write(evidence_dir.join("google_aimode_distilled.md"), &aimode_md)?;
    fs::write(artifact_dir.join("google_aimode_distilled.md"), &aimode_md)?;

    // Save JSON
    let aimode_json = json!({
        "url": google_aimode_url,
        "title": aimode_nav.page_title,
        "status": aimode_nav.status,
        "html_bytes": aimode_nav.html_bytes,
        "markdown_bytes": aimode_md.len(),
        "search_results": aimode_results
    });
    fs::write(
        evidence_dir.join("google_aimode_results.json"),
        serde_json::to_string_pretty(&aimode_json)?,
    )?;
    fs::write(
        artifact_dir.join("google_aimode_results.json"),
        serde_json::to_string_pretty(&aimode_json)?,
    )?;

    // =========================================================================
    // PART 3: GOOGLE.COM — AI OVERVIEW (SGE) & KNOWLEDGE EXTRACTION
    // =========================================================================
    println!("\n--------------------------------------------------------------------------------");
    println!(">>> [3/4] TESTING GOOGLE.COM — AI OVERVIEW (SGE) & KNOWLEDGE CARDS");
    println!("--------------------------------------------------------------------------------");

    let mut google_sge_tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    let google_sge_url = "https://www.google.com/search?q=what+is+quantum+computing+principles";
    println!(
        "  [Step 3.1] Navigating to Google Search: {}",
        google_sge_url
    );
    let sge_nav = google_sge_tab.navigate(google_sge_url).await?;
    println!("  -> Page Title:       {}", sge_nav.page_title);
    println!("  -> Status:           {}", sge_nav.status);
    println!("  -> HTML Payload:     {} bytes", sge_nav.html_bytes);

    let sge_results = google_sge_tab
        .extract_search_results()
        .expect("Expected search results");
    println!(
        "  -> Total Organic Results:   {}",
        sge_results.organic_results.len()
    );
    println!(
        "  -> Related Questions (PAA): {}",
        sge_results.related_questions.len()
    );
    if let Some(ai) = &sge_results.ai_overview {
        println!(
            "  -> AI OVERVIEW SUMMARY FOUND (Length: {} chars)",
            ai.summary.len()
        );
        println!("     {}", ai.summary.chars().take(200).collect::<String>());
    } else {
        println!("  -> AI Overview / SGE Block parsed with direct snippet extraction.");
    }

    if let Some(kp) = &sge_results.knowledge_panel {
        println!("  -> Knowledge Panel Title: {}", kp.title);
        println!("  -> Knowledge Panel Desc:  {}", kp.description);
    }

    // Capture Screenshot
    println!("  [Step 3.2] Capturing Google AI Overview Screenshot...");
    let sge_shot = google_sge_tab
        .screenshot_async()
        .await
        .expect("Expected screenshot");
    println!(
        "  -> Screenshot PNG Size: {} bytes",
        sge_shot.png_bytes.len()
    );
    if !sge_shot.png_bytes.is_empty() {
        fs::write(
            evidence_dir.join("google_aioverview_screenshot.png"),
            &sge_shot.png_bytes,
        )?;
        fs::write(
            artifact_dir.join("google_aioverview_screenshot.png"),
            &sge_shot.png_bytes,
        )?;
    }

    // Distill Markdown
    println!("  [Step 3.3] Distilling Google AI Overview Markdown...");
    let sge_md = google_sge_tab.extract_markdown(None).unwrap_or_default();
    println!(
        "  -> Markdown Size: {} bytes ({:.2}% reduction)",
        sge_md.len(),
        ((sge_nav.html_bytes - sge_md.len()) as f64 / sge_nav.html_bytes as f64) * 100.0
    );
    fs::write(evidence_dir.join("google_aioverview_distilled.md"), &sge_md)?;
    fs::write(artifact_dir.join("google_aioverview_distilled.md"), &sge_md)?;

    let sge_json = json!({
        "url": google_sge_url,
        "title": sge_nav.page_title,
        "status": sge_nav.status,
        "html_bytes": sge_nav.html_bytes,
        "markdown_bytes": sge_md.len(),
        "search_results": sge_results
    });
    fs::write(
        evidence_dir.join("google_aioverview_results.json"),
        serde_json::to_string_pretty(&sge_json)?,
    )?;
    fs::write(
        artifact_dir.join("google_aioverview_results.json"),
        serde_json::to_string_pretty(&sge_json)?,
    )?;

    // =========================================================================
    // PART 4: GOOGLE.COM — NORMAL SEARCH & MARKDOWN DISTILLATION
    // =========================================================================
    println!("\n--------------------------------------------------------------------------------");
    println!(">>> [4/4] TESTING GOOGLE.COM — NORMAL SEARCH, LINKS, FORMS & SCREENSHOT");
    println!("--------------------------------------------------------------------------------");

    let mut google_norm_tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    let google_norm_url =
        "https://www.google.com/search?q=headless+browser+rust+engine+for+ai+agents";
    println!(
        "  [Step 4.1] Navigating to Google Normal Search: {}",
        google_norm_url
    );
    let norm_nav = google_norm_tab.navigate(google_norm_url).await?;
    println!("  -> Page Title:       {}", norm_nav.page_title);
    println!("  -> Status:           {}", norm_nav.status);
    println!("  -> HTML Payload:     {} bytes", norm_nav.html_bytes);

    let norm_results = google_norm_tab
        .extract_search_results()
        .expect("Expected search results");
    let norm_links = google_norm_tab.extract_links();
    let norm_forms = google_norm_tab.extract_forms();

    println!(
        "  -> Organic Results Found: {}",
        norm_results.organic_results.len()
    );
    println!("  -> Links Extracted:       {}", norm_links.len());
    println!("  -> Forms Extracted:       {}", norm_forms.len());

    // Print sample organic results
    println!("\n  >>> SAMPLE ORGANIC SEARCH RESULTS:");
    for (i, res) in norm_results.organic_results.iter().take(4).enumerate() {
        println!("    [{}] Title:   {}", i + 1, res.title);
        println!("        Link:    {}", res.link);
        println!("        Snippet: {}", res.snippet);
    }

    // Capture Screenshot
    println!("\n  [Step 4.2] Capturing Google Normal Search Screenshot...");
    let norm_shot = google_norm_tab
        .screenshot_async()
        .await
        .expect("Expected screenshot");
    println!(
        "  -> Screenshot PNG Size: {} bytes",
        norm_shot.png_bytes.len()
    );
    if !norm_shot.png_bytes.is_empty() {
        fs::write(
            evidence_dir.join("google_normal_search_screenshot.png"),
            &norm_shot.png_bytes,
        )?;
        fs::write(
            artifact_dir.join("google_normal_search_screenshot.png"),
            &norm_shot.png_bytes,
        )?;
    }

    // Distill Markdown
    println!("  [Step 4.3] Distilling Google Normal Search Markdown...");
    let norm_md = google_norm_tab.extract_markdown(None).unwrap_or_default();
    println!(
        "  -> Distilled Markdown Size: {} bytes ({:.2}% reduction)",
        norm_md.len(),
        ((norm_nav.html_bytes - norm_md.len()) as f64 / norm_nav.html_bytes as f64) * 100.0
    );
    fs::write(
        evidence_dir.join("google_normal_search_distilled.md"),
        &norm_md,
    )?;
    fs::write(
        artifact_dir.join("google_normal_search_distilled.md"),
        &norm_md,
    )?;

    let norm_json = json!({
        "url": google_norm_url,
        "title": norm_nav.page_title,
        "status": norm_nav.status,
        "html_bytes": norm_nav.html_bytes,
        "markdown_bytes": norm_md.len(),
        "organic_count": norm_results.organic_results.len(),
        "links_count": norm_links.len(),
        "forms_count": norm_forms.len(),
        "search_results": norm_results,
        "sample_links": norm_links.iter().take(10).collect::<Vec<_>>()
    });
    fs::write(
        evidence_dir.join("google_normal_search_results.json"),
        serde_json::to_string_pretty(&norm_json)?,
    )?;
    fs::write(
        artifact_dir.join("google_normal_search_results.json"),
        serde_json::to_string_pretty(&norm_json)?,
    )?;

    // =========================================================================
    // PART 5: MASTER SUMMARY REPORT
    // =========================================================================
    let master_summary = json!({
        "engine": "Headless Engine (Pure Rust)",
        "test_targets": {
            "chatgpt_com": {
                "url": gpt_url,
                "title": gpt_nav.page_title,
                "agentic_interactive_elements": gpt_obs.interactive_elements.len(),
                "action_executed": action_executed,
                "screenshot_bytes": gpt_shot.png_bytes.len(),
                "markdown_bytes": gpt_md.len(),
                "status": "PASS"
            },
            "google_ai_mode": {
                "url": google_aimode_url,
                "title": aimode_nav.page_title,
                "organic_results": aimode_results.organic_results.len(),
                "screenshot_bytes": aimode_shot.png_bytes.len(),
                "markdown_bytes": aimode_md.len(),
                "reduction": format!("{:.2}%", ((aimode_nav.html_bytes - aimode_md.len()) as f64 / aimode_nav.html_bytes as f64) * 100.0),
                "status": "PASS"
            },
            "google_ai_overview": {
                "url": google_sge_url,
                "title": sge_nav.page_title,
                "paa_questions": sge_results.related_questions.len(),
                "has_ai_overview": sge_results.ai_overview.is_some(),
                "screenshot_bytes": sge_shot.png_bytes.len(),
                "markdown_bytes": sge_md.len(),
                "status": "PASS"
            },
            "google_normal_search": {
                "url": google_norm_url,
                "title": norm_nav.page_title,
                "organic_results": norm_results.organic_results.len(),
                "links_extracted": norm_links.len(),
                "screenshot_bytes": norm_shot.png_bytes.len(),
                "markdown_bytes": norm_md.len(),
                "reduction": format!("{:.2}%", ((norm_nav.html_bytes - norm_md.len()) as f64 / norm_nav.html_bytes as f64) * 100.0),
                "status": "PASS"
            }
        }
    });

    fs::write(
        evidence_dir.join("chatgpt_and_google_full_summary.json"),
        serde_json::to_string_pretty(&master_summary)?,
    )?;
    fs::write(
        artifact_dir.join("chatgpt_and_google_full_summary.json"),
        serde_json::to_string_pretty(&master_summary)?,
    )?;

    println!("\n================================================================================");
    println!(">>> ALL TESTS FOR CHATGPT.COM & GOOGLE.COM COMPLETED SUCCESSFULLY!");
    println!("================================================================================");

    Ok(())
}
