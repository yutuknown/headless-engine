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
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(
    name = "headless-engine",
    author = "Headless Engine Contributors",
    version = "1.0.0",
    about = "Ultra-lightweight (<30MB RAM), detection-free pure-Rust headless browser engine for AI agents, web scraping, and MCP servers."
)]
struct Cli {
    /// URL to navigate, search query, or interactive command
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

    /// Start interactive Claude Code-style REPL session
    #[arg(short, long)]
    interactive: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let profile = parse_profile(&cli.profile);

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

    // Mode 2: Interactive REPL Mode (if no target given or --interactive passed)
    if cli.interactive || (cli.target.is_none() && !cli.markdown && !cli.search && !cli.json) {
        return run_interactive_repl(profile, cli.proxy, cli.timeout).await;
    }

    // Mode 3: Direct CLI Execution
    let target = cli.target.unwrap_or_else(|| "https://news.ycombinator.com".to_string());
    execute_target(&target, profile, cli.proxy.as_deref(), cli.timeout, cli.markdown, cli.search, cli.json).await
}

fn parse_profile(s: &str) -> DeviceProfile {
    match s.to_lowercase().as_str() {
        "chrome-linux" | "linux" => DeviceProfile::ChromeLinux,
        "safari-mac" | "mac" | "safari" => DeviceProfile::SafariMac,
        "safari-ios" | "ios" | "iphone" => DeviceProfile::SafariIos,
        "chrome-android" | "android" => DeviceProfile::ChromeAndroid,
        _ => DeviceProfile::ChromeWindows,
    }
}

async fn execute_target(
    target: &str,
    profile: DeviceProfile,
    proxy: Option<&str>,
    timeout_secs: u64,
    markdown_only: bool,
    search_mode: bool,
    json_mode: bool,
) -> anyhow::Result<()> {
    let mut builder = BrowserBuilder::new()
        .profile(profile)
        .timeout(Duration::from_secs(timeout_secs));

    if let Some(proxy_str) = proxy {
        builder = builder.proxy(proxy_str);
    }

    let mut tab = builder.build()?;

    let target_url = if search_mode && !target.starts_with("http://") && !target.starts_with("https://") {
        format!("https://www.google.com/search?q={}", urlencoding_simple(target))
    } else {
        target.to_string()
    };

    let start = Instant::now();
    let report = tab.navigate(&target_url).await?;
    let duration = start.elapsed();

    if markdown_only {
        let md = tab.extract_markdown(None).unwrap_or_default();
        if json_mode {
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

    if search_mode {
        if let Some(search_results) = tab.extract_search_results() {
            println!("{}", serde_json::to_string_pretty(&search_results)?);
        } else {
            println!("{{ \"error\": \"No search results found\" }}");
        }
        return Ok(());
    }

    if json_mode {
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

    // Claude Code-style UX rendering
    let markdown = tab.extract_markdown(None).unwrap_or_default();
    let links = tab.extract_links();
    let forms = tab.extract_forms();

    let compression = if report.html_bytes > 0 {
        (1.0 - (markdown.len() as f64 / report.html_bytes as f64)) * 100.0
    } else {
        0.0
    };

    let status_badge = if report.status >= 200 && report.status < 300 {
        format!("\x1b[1;32m{} OK\x1b[0m", report.status)
    } else {
        format!("\x1b[1;33m{} Redirect/Warn\x1b[0m", report.status)
    };

    let profile_name = format!("{:?}", profile).to_lowercase();

    // Top Card
    eprintln!("\x1b[38;5;240m╭─\x1b[0m \x1b[1m{}\x1b[0m \x1b[38;5;244m({})\x1b[0m", report.final_url, profile_name);
    eprintln!(
        "\x1b[38;5;240m│\x1b[0m  \x1b[38;5;39m●\x1b[0m \x1b[1mNetwork\x1b[0m    {} \x1b[38;5;240m·\x1b[0m \x1b[38;5;244m{:.2} KB\x1b[0m \x1b[38;5;240m·\x1b[0m \x1b[38;5;244m{:?}\x1b[0m",
        status_badge,
        report.html_bytes as f64 / 1024.0,
        duration
    );
    eprintln!(
        "\x1b[38;5;240m│\x1b[0m  \x1b[38;5;39m●\x1b[0m \x1b[1mDOM Engine\x1b[0m \x1b[38;5;244m{} actionable links\x1b[0m \x1b[38;5;240m·\x1b[0m \x1b[38;5;244m{} interactive forms\x1b[0m",
        links.len(),
        forms.len()
    );
    eprintln!(
        "\x1b[38;5;240m│\x1b[0m  \x1b[38;5;39m●\x1b[0m \x1b[1mMarkdown\x1b[0m   \x1b[38;5;244m{} bytes\x1b[0m \x1b[38;5;240m·\x1b[0m \x1b[1;32m{:.1}% token compression\x1b[0m",
        markdown.len(),
        compression.max(0.0)
    );
    if !report.page_title.is_empty() {
        eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[38;5;39m●\x1b[0m \x1b[1mTitle\x1b[0m      \x1b[38;5;250m{}\x1b[0m", report.page_title);
    }
    eprintln!("\x1b[38;5;240m╰─\x1b[0m \x1b[38;5;244mCompleted in {:?}\x1b[0m\n", duration);

    // Markdown Preview Box
    let preview_lines: Vec<&str> = markdown.lines().filter(|l| !l.trim().is_empty()).take(12).collect();
    if !preview_lines.is_empty() {
        eprintln!("\x1b[38;5;240m╭─\x1b[0m \x1b[1mLLM Markdown Content\x1b[0m \x1b[38;5;244m(first {} lines)\x1b[0m", preview_lines.len());
        for line in preview_lines {
            let truncated = if line.len() > 100 {
                format!("{}...", &line[..97])
            } else {
                line.to_string()
            };
            eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[38;5;252m{}\x1b[0m", truncated);
        }
        eprintln!("\x1b[38;5;240m╰─\x1b[0m \x1b[38;5;244mRun with -m / --markdown to dump complete payload\x1b[0m\n");
    }

    Ok(())
}

async fn run_interactive_repl(
    mut current_profile: DeviceProfile,
    proxy: Option<String>,
    timeout_secs: u64,
) -> anyhow::Result<()> {
    eprintln!("\x1b[38;5;240m╭─\x1b[0m \x1b[1;38;5;39mheadless-engine\x1b[0m \x1b[38;5;244mv1.0.0\x1b[0m");
    eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[38;5;248mPure-Rust Headless Browser Engine for AI Agents & Scraping\x1b[0m");
    eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[38;5;244mType a URL, search query, or /help for commands\x1b[0m");
    eprintln!("\x1b[38;5;240m╰────────────────────────────────────────────────────────────\x1b[0m\n");

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    loop {
        eprint!("\x1b[1;38;5;39m❯\x1b[0m ");
        io::stderr().flush()?;

        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" || input == "/exit" || input == "/quit" || input == "q" {
            eprintln!("\x1b[38;5;244mSession ended.\x1b[0m");
            break;
        }

        if input == "/clear" || input == "clear" || input == "cls" {
            eprint!("\x1b[2J\x1b[1;1H");
            continue;
        }

        if input == "/help" || input == "help" || input == "?" {
            eprintln!("\x1b[38;5;240m╭─\x1b[0m \x1b[1mAvailable Commands\x1b[0m");
            eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[1m<url>\x1b[0m                     Navigate and extract LLM Markdown & elements");
            eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[1m/search <query>\x1b[0m           Perform search & extract SERP entities");
            eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[1m/profile <name>\x1b[0m           Switch device [chrome-windows, safari-ios, android, mac]");
            eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[1m/markdown <url>\x1b[0m           Dump complete raw Markdown to stdout");
            eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[1m/clear\x1b[0m                    Clear terminal window");
            eprintln!("\x1b[38;5;240m│\x1b[0m  \x1b[1m/exit\x1b[0m                     Quit interactive session");
            eprintln!("\x1b[38;5;240m╰─────────────────────────────────────────────────────────────\x1b[0m\n");
            continue;
        }

        if input.starts_with("/profile ") {
            let p_str = input["/profile ".len()..].trim();
            current_profile = parse_profile(p_str);
            eprintln!("\x1b[32m✓\x1b[0m Active profile set to: \x1b[1m{:?}\x1b[0m\n", current_profile);
            continue;
        }

        let is_search = input.starts_with("/search ") || input.starts_with("search ");
        let is_markdown = input.starts_with("/markdown ") || input.starts_with("md ");

        let target = if is_search {
            input.split_once(' ').map(|x| x.1).unwrap_or(input)
        } else if is_markdown {
            input.split_once(' ').map(|x| x.1).unwrap_or(input)
        } else {
            input
        };

        let target_url = if !target.starts_with("http://") && !target.starts_with("https://") {
            if is_search {
                format!("https://www.google.com/search?q={}", urlencoding_simple(target))
            } else {
                format!("https://{}", target)
            }
        } else {
            target.to_string()
        };

        if let Err(e) = execute_target(
            &target_url,
            current_profile,
            proxy.as_deref(),
            timeout_secs,
            is_markdown,
            is_search,
            false,
        )
        .await
        {
            eprintln!("\x1b[1;31m✗ Error:\x1b[0m {}\n", e);
        }
    }

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
