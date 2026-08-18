use anyhow::Result;
use headless_engine::browser::tab::BrowserTab;
use headless_engine::network::fingerprint::DeviceProfile;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let query_or_url = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "https://en.wikipedia.org/wiki/Quantum_computing".to_string()
    };

    // Minimalistic timestamp generator for structured logging
    let time_fmt = || {
        let now = std::time::SystemTime::now();
        let secs = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let h = (secs / 3600) % 24;
        let m = (secs / 60) % 60;
        let s = secs % 60;
        format!("\x1b[90m{:02}:{:02}:{:02}\x1b[0m", h, m, s)
    };

    macro_rules! step {
        ($($arg:tt)*) => {
            eprintln!("{} \x1b[34m::\x1b[0m {}", time_fmt(), format!($($arg)*));
        };
    }
    macro_rules! success {
        ($($arg:tt)*) => {
            eprintln!("{} \x1b[32m✓\x1b[0m  {}", time_fmt(), format!($($arg)*));
        };
    }
    macro_rules! warn {
        ($($arg:tt)*) => {
            eprintln!("{} \x1b[33m!\x1b[0m  {}", time_fmt(), format!($($arg)*));
        };
    }

    step!("Initializing headless session...");
    let mut tab = BrowserTab::new()?;

    step!("Navigating to target: \x1b[36m{}\x1b[0m", query_or_url);
    let nav_info = tab.navigate(&query_or_url).await?;

    let status_color = if nav_info.status == 200 { "\x1b[32m" } else { "\x1b[33m" };
    success!("Navigation complete [Status: {}{}\x1b[0m | Size: \x1b[36m{:.2} MB\x1b[0m]", 
        status_color, nav_info.status, nav_info.html_bytes as f64 / 1_048_576.0);

    if nav_info.is_captcha_detected {
        warn!("Captcha challenge detected on target page");
    }

    step!("Inspecting JS execution environment...");
    let _user_agent = tab.evaluate_js("navigator.userAgent")?.replace("\"", "");
    let webdriver = tab.evaluate_js("navigator.webdriver")?.replace("\"", "");
    let platform = tab.evaluate_js("navigator.platform")?.replace("\"", "");
    success!("Environment verified (Platform: \x1b[36m{}\x1b[0m, Webdriver: \x1b[36m{}\x1b[0m)", platform, webdriver);

    step!("Extracting structural DOM node descriptors...");
    let links = tab.extract_links();
    let forms = tab.extract_forms();
    
    if let Some(search_results) = tab.extract_search_results() {
        if search_results.total_results_found > 0 {
            success!("Extracted \x1b[36m{}\x1b[0m links, \x1b[36m{}\x1b[0m forms, and \x1b[36m{}\x1b[0m search entities (\x1b[90m{}\x1b[0m)", 
                links.len(), forms.len(), search_results.total_results_found, search_results.page_title);
        } else {
            success!("Extracted \x1b[36m{}\x1b[0m links and \x1b[36m{}\x1b[0m interactive forms", links.len(), forms.len());
        }
    } else {
        success!("Extracted \x1b[36m{}\x1b[0m links and \x1b[36m{}\x1b[0m interactive forms", links.len(), forms.len());
    }

    step!("Converting DOM tree to LLM-optimized Markdown...");
    let markdown = tab.extract_markdown(None).unwrap_or_default();
    let compression = (1.0 - (markdown.len() as f64 / nav_info.html_bytes as f64)) * 100.0;
    success!("Payload compressed to \x1b[36m{}\x1b[0m bytes (\x1b[32m~{:.1}%\x1b[0m reduction)", markdown.len(), compression);

    step!("Demonstrating runtime profile mutation (Safari iOS)...");
    tab.set_profile(DeviceProfile::SafariIos)?;
    let ios_nav = tab.navigate("https://news.ycombinator.com/").await?;
    success!("iOS spoof complete [Status: \x1b[32m{}\x1b[0m]", ios_nav.status);

    step!("Session complete. Dumping markdown payload to stdout.");
    eprintln!("\x1b[90m{}\x1b[0m", "-".repeat(80));
    
    // Pure Unix pipeline compliance: Print raw markdown payload to stdout
    println!("{}", markdown);

    Ok(())
}
