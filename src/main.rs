#![allow(dead_code)]

mod api;
mod browser;
mod dom;
mod js;
mod network;
mod render;

use api::rpc::JsonRpcHandler;
use browser::builder::BrowserBuilder;
use clap::Parser;
use network::fingerprint::DeviceProfile;
use std::io::{self, BufRead, Write};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "headless-engine",
    author = "Headless Engine Contributors",
    version = "0.2.0",
    about = "Ultra-lightweight (<30MB RAM), detection-free pure-Rust headless browser engine for AI agents, web scraping, and MCP servers."
)]
struct Cli {
    /// URL to navigate or query to search
    #[arg(value_name = "TARGET")]
    target: Option<String>,

    /// Run as JSON-RPC 2.0 server over stdin/stdout (MCP agent mode)
    #[arg(long)]
    stdio: bool,

    /// Extract clean, token-efficient LLM Markdown directly to stdout
    #[arg(short, long)]
    markdown: bool,

    /// Perform multi-modal search and output structured JSON
    #[arg(short, long)]
    search: bool,

    /// Device profile to emulate [chrome-windows, chrome-linux, safari-mac, safari-ios, chrome-android]
    #[arg(short, long, default_value = "chrome-windows")]
    profile: String,

    /// HTTP/HTTPS/SOCKS5 proxy URL (e.g. socks5://127.0.0.1:9050 or http://user:pass@proxy:8080)
    #[arg(long)]
    proxy: Option<String>,

    /// Timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Print output in pretty-printed JSON format
    #[arg(short, long)]
    json: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let profile = match cli.profile.to_lowercase().as_str() {
        "chrome-linux" | "linux" => DeviceProfile::ChromeLinux,
        "safari-mac" | "mac" | "safari" => DeviceProfile::SafariMac,
        "safari-ios" | "ios" | "iphone" => DeviceProfile::SafariIos,
        "chrome-android" | "android" => DeviceProfile::ChromeAndroid,
        _ => DeviceProfile::ChromeWindows,
    };

    // Mode 1: JSON-RPC 2.0 STDIO Mode for Go MCP servers and AI agents
    if cli.stdio {
        let mut handler = JsonRpcHandler::new()?;
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let response = handler.handle_line(&line).await;
            let is_shutdown = line.contains("\"method\":\"shutdown\"")
                || line.contains("\"method\":\"Shutdown\"");

            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;

            if is_shutdown {
                break;
            }
        }
        return Ok(());
    }

    // Mode 2: Direct CLI Mode
    let target = cli
        .target
        .unwrap_or_else(|| "https://en.wikipedia.org/wiki/Artificial_intelligence".to_string());

    let mut builder = BrowserBuilder::new()
        .profile(profile)
        .timeout(Duration::from_secs(cli.timeout));

    if let Some(proxy_str) = cli.proxy {
        builder = builder.proxy(proxy_str);
    }

    let mut tab = builder.build()?;

    let target_url =
        if cli.search && !target.starts_with("http://") && !target.starts_with("https://") {
            format!(
                "https://www.google.com/search?q={}",
                urlencoding_simple(&target)
            )
        } else {
            target
        };

    let report = tab.navigate(&target_url).await?;

    if cli.markdown {
        let md = tab.extract_markdown(None).unwrap_or_default();
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "url": report.final_url,
                    "title": report.page_title,
                    "markdown": md
                }))?
            );
        } else {
            println!("{}", md);
        }
        return Ok(());
    }

    if cli.search {
        if let Some(search_results) = tab.extract_search_results() {
            println!("{}", serde_json::to_string_pretty(&search_results)?);
        } else {
            println!("{{ \"error\": \"No search results found\" }}");
        }
        return Ok(());
    }

    if cli.json {
        let md = tab.extract_markdown(None).unwrap_or_default();
        let links = tab.extract_links();
        let forms = tab.extract_forms();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "navigation": report,
                "markdown": md,
                "links_count": links.len(),
                "forms_count": forms.len(),
                "links": links.into_iter().take(20).collect::<Vec<_>>(),
                "forms": forms
            }))?
        );
        return Ok(());
    }

    // Minimalist logging (Lightpanda UX style, professional ASCII)
    let time_fmt = || {
        let now = std::time::SystemTime::now();
        let secs = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
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

    let status_color = if report.status == 200 {
        "\x1b[32m"
    } else {
        "\x1b[33m"
    };
    success!(
        "Navigation complete [Status: {}{}\x1b[0m | Target: \x1b[36m{}\x1b[0m]",
        status_color,
        report.status,
        report.final_url
    );

    step!("Page Title: \x1b[36m{}\x1b[0m", report.page_title);
    success!(
        "Transferred: \x1b[36m{:.2} MB\x1b[0m",
        report.html_bytes as f64 / 1_048_576.0
    );

    if report.is_captcha_detected {
        warn!("Captcha challenge detected on target page");
    }

    let markdown = tab.extract_markdown(None).unwrap_or_default();
    let compression = (1.0 - (markdown.len() as f64 / report.html_bytes as f64)) * 100.0;
    success!("Extracted \x1b[36m{}\x1b[0m bytes of markdown (\x1b[32m~{:.1}%\x1b[0m token compression ratio)", markdown.len(), compression);

    let links = tab.extract_links();
    let forms = tab.extract_forms();
    success!(
        "Extracted \x1b[36m{}\x1b[0m actionable links and \x1b[36m{}\x1b[0m interactive forms",
        links.len(),
        forms.len()
    );

    step!("Use --markdown to dump raw payload, --search for SERP JSON, or --stdio for MCP.");

    Ok(())
}

fn urlencoding_simple(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
