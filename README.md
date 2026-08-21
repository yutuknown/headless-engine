<div align="center">
  <img src="https://raw.githubusercontent.com/yutuknown/headless-engine/master/assets/logo.svg" alt="Headless Engine Logo" width="460">
  <br><br>

  <p><strong>A high-performance, lightweight (&lt;30MB RAM) headless browser engine written in Rust for AI agents, web automation, and LLM data ingestion.</strong></p>

  <p>
    <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-stable-orange.svg" alt="Rust"></a>
    <a href="https://crates.io/crates/headless-engine"><img src="https://img.shields.io/crates/v/headless-engine.svg" alt="crates.io"></a>
    <a href="https://pypi.org/project/headless-engine/"><img src="https://img.shields.io/pypi/v/headless-engine.svg" alt="PyPI"></a>
    <a href="https://www.npmjs.com/package/headless-engine"><img src="https://img.shields.io/npm/v/headless-engine.svg" alt="npm"></a>
    <img src="https://img.shields.io/badge/RAM-~9.4MB_idle-brightgreen.svg" alt="Memory">
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License"></a>
  </p>
</div>

Traditional headless browsers (such as Puppeteer or Playwright) orchestrate full multi-process Chromium instances that typically consume 350MB–800MB of RAM per instance with significant initialization overhead.

**Headless Engine** is designed as a purpose-built alternative: a fast, pure-Rust headless browser runtime that operates with a **<30MB RSS footprint** and sub-5ms initialization. It parses DOM trees directly, applies client fingerprint emulation to avoid automated bot triggers, and converts pages into clean, token-efficient Markdown optimized for LLM context windows.

---

## Key Features

- 🪶 **Low Memory Overhead:** Sub-10MB idle RSS and ~15–35MB under active browsing, enabling high-density concurrency on resource-constrained servers.
- 🛡️ **Fingerprint Emulation & Anti-Detection:** Injects realistic client profiles (`navigator.webdriver = false`, full `window.chrome` hierarchy, WebGL ANGLE vendor/renderer masking, Client Hints, and session persistence).
- 🤖 **Search & Knowledge Extraction:** Built-in extraction for Google AI Mode (`udm=50`), AI Overviews (SGE), Oxford Languages Knowledge Cards, YouTube video timestamps, and organic results.
- 📝 **LLM-Ready Markdown Distillation:** Natively converts HTML DOM trees into clean Markdown by stripping navigation menus, ads, headers/footers, and boilerplate scripts (~85% token reduction).
- ⚡ **Offline Ingestion & DOM Injection:** Ingest raw HTML strings or local files into active tabs via `browser.set_content(html)` or `file://` URIs for offline parsing and deterministic testing.
- 📱 **Multi-Device Profiles:** Switch between Windows Chrome, Linux Chrome, macOS Safari, iOS Safari, and Android Chrome.
- 🔌 **Universal Multi-Language Support:** First-class SDKs and bindings for **Rust**, **Python**, **Node.js / TypeScript**, **Go**, and **Docker**.
- 🗂️ **Multi-Tab Isolation:** Built-in arena-allocated tab manager (`BrowserEngine`) for concurrent, isolated multi-tab automation.
- 📡 **Standard JSON-RPC 2.0 Interface:** Connect over standard I/O for direct integration with AI agents and Model Context Protocol (MCP) servers.

---

## 🚀 31 Multimodal Google & YouTube Capabilities

In `v1.1.0`, we've introduced direct API integrations for 31 specific search and media modalities across Google and YouTube. These endpoints use a **Tiered Stealth Architecture**: defaulting to pure-Rust HTTP/2 extraction for ~18ms latency, with an automated ephemeral compositor fallback (`--dump-dom`) if captchas are encountered.

Each capability yields **Dual Outputs**:
1. **Strongly Typed Structs** (e.g. `GoogleSearchResult`, `GoogleAutocompleteResult`) for programmatic extraction.
2. **LLM Distillation**: A `.to_markdown()` method optimized for massive (80-95%) token savings.

### Supported Endpoints

- **General Search**: `google_search`, `google_web_search`, `google_autocomplete`
- **AI & Vertical**: `google_ai_overview`, `google_ai_mode`
- **Media & Products**: `google_image_search`, `google_video_search`, `google_short_video_search`, `google_news_search`, `google_forum_search`, `google_shopping_search`, `google_product_search`, `google_books_search`
- **Travel & Maps**: `google_maps_search`, `google_flights_search`, `google_hotels_search`, `google_travel_explore`
- **Finance & Academics**: `google_finance_quote`, `google_scholar_search`, `google_patents_search`, `google_trends_search`
- **YouTube Ecosystem**: `youtube_search`, `youtube_shorts_search`, `youtube_video`, `youtube_channel`, `youtube_playlist`
- **Google Lens**: `google_lens_visual_matches`, `google_lens_exact_matches`, `google_lens_products`, `google_lens_about_image`

**Example (Rust):**
```rust
let mut tab = BrowserTab::new()?;
let flight_results = tab.google_flights_search("SFO", "JFK").await?;
println!("{}", flight_results.to_markdown());
```

**Example (Python SDK via RPC):**
```python
res = engine.call_rpc("tab.google_finance_quote", {"tab_id": tab_id, "query": "AAPL"})
print(res.get("raw_markdown"))
```

---

## Installation & Quick Start

### 1. 🦀 Rust Crate
```bash
cargo add headless-engine
```
```rust
use headless_engine::{BrowserTab, DeviceProfile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut tab = BrowserTab::with_profile(DeviceProfile::ChromeWindows)?;
    let report = tab.navigate("https://news.ycombinator.com/").await?;
    
    println!("Title: {} (Status: {})", report.page_title, report.status);
    let markdown = tab.extract_markdown(None).unwrap();
    println!("Markdown:\n{}", markdown);
    
    Ok(())
}
```

### 2. 🐍 Python SDK (`pip`)
```bash
pip install headless-engine
```
```python
from headless_engine import HeadlessBrowser

with HeadlessBrowser() as browser:
    # Navigate with automated fingerprint emulation
    report = browser.navigate("https://www.google.com/search?q=quantum+computing&udm=50")
    
    # Extract token-efficient LLM Markdown
    markdown = browser.extract_markdown()
    print("Markdown:\n", markdown)
    
    # Extract structured search entities (AI Overviews, PAA, Organic Results)
    results = browser.extract_results()
```

### 3. 🟢 Node.js / TypeScript SDK (`npm`)
```bash
npm install headless-engine
```
```typescript
import { HeadlessBrowser } from 'headless-engine';

const browser = new HeadlessBrowser();
const report = await browser.navigate('https://news.ycombinator.com');
const markdown = await browser.extractMarkdown();
console.log('Markdown:', markdown);
browser.close();
```

### 4. 🐹 Go Module
```bash
go get github.com/yutuknown/headless-engine/sdk/go
```
```go
package main

import (
    "fmt"
    "github.com/yutuknown/headless-engine/sdk/go"
)

func main() {
    client, _ := headless.NewClient("")
    defer client.Close()

    report, _ := client.Navigate("https://en.wikipedia.org/wiki/Quantum_computing")
    markdown, _ := client.ExtractMarkdown("")
    fmt.Printf("Page Title: %s\nContent Length: %d\n", report.PageTitle, len(markdown))
}
```

### 5. 🐳 Docker Container
```bash
docker run -d --name headless-engine -p 9222:9222 ghcr.io/yutuknown/headless-engine:latest
```

### 6. ⚡ Standalone Binary Installers
- **Linux & macOS:**
  ```bash
  curl -fsSL https://raw.githubusercontent.com/yutuknown/headless-engine/master/scripts/install.sh | bash
  ```
- **Windows PowerShell:**
  ```powershell
  iwr -useb https://raw.githubusercontent.com/yutuknown/headless-engine/master/scripts/install.ps1 | iex
  ```

---

## Architectural Comparison

| Metric / Capability | Chromium (Playwright/Puppeteer) | Lightpanda (Zig) | **Headless Engine (Rust)** |
| :--- | :--- | :--- | :--- |
| **Idle Memory (RSS)** | ~120 MB | ~20 MB | **~9.4 MB** |
| **Active Memory (1 Tab)** | ~350 MB – 600 MB | ~48 MB | **~15 MB – 32 MB** |
| **Active Memory (5 Tabs)** | ~800 MB – 1.8 GB | ~180 MB | **~35.6 MB** |
| **Native Windows Support** | ✅ Yes | ❌ WSL2 Required | **✅ Native Windows `.exe` + Linux + macOS** |
| **Startup Latency** | ~800 ms – 1,500 ms | ~40 ms | **< 5 ms** |
| **LLM Markdown Distillation** | Requires third-party library | Basic HTML dump | **Native AST Converter (~85% Token Reduction)** |
| **Structured SERP Parsing** | Manual scraping required | None | **Built-in (AI Overviews, PAA, Knowledge Cards)** |
| **Inter-Process Protocol** | Chrome DevTools Protocol (CDP) | Partial CDP | **Standard JSON-RPC 2.0 (Stdio / MCP)** |

---

## 🤖 MCP Server & CLI Automation Example

Headless Engine provides a standard JSON-RPC 2.0 interface over Stdio:

```go
package main

import (
    "bufio"
    "encoding/json"
    "fmt"
    "os/exec"
)

func main() {
    cmd := exec.Command("headless-engine", "--stdio")
    stdin, _ := cmd.StdinPipe()
    stdout, _ := cmd.StdoutPipe()
    cmd.Start()
    scanner := bufio.NewScanner(stdout)

    // 1. Send Navigate Command
    req, _ := json.Marshal(map[string]interface{}{
        "jsonrpc": "2.0",
        "id":      1,
        "method":  "tab.navigate",
        "params":  map[string]string{"url": "https://en.wikipedia.org/wiki/Artificial_intelligence"},
    })
    stdin.Write(append(req, '\n'))
    scanner.Scan()
    fmt.Println("Navigation Response:", scanner.Text())

    // 2. Extract LLM Markdown
    req2, _ := json.Marshal(map[string]interface{}{
        "jsonrpc": "2.0",
        "id":      2,
        "method":  "tab.extractMarkdown",
    })
    stdin.Write(append(req2, '\n'))
    scanner.Scan()
    fmt.Println("LLM Markdown:", scanner.Text())

    cmd.Process.Kill()
}
```

---

## 📚 JSON-RPC 2.0 API Reference

| Method | Parameters | Description |
| :--- | :--- | :--- |
| `tab.navigate` | `{ "url": "...", "tab_id": "..." }` | Navigates to target URL with fingerprint emulation |
| `tab.setContent` | `{ "html": "...", "url": "...", "tab_id": "..." }` | Injects raw HTML into the tab for offline parsing |
| `tab.extractMarkdown` | `{ "selector": "...", "tab_id": "..." }` | Returns filtered, token-efficient LLM Markdown |
| `tab.extractResults` | `{ "tab_id": "..." }` | Returns structured search entities (AI Overviews, PAA, results) |
| `tab.extractLinks` | `{ "tab_id": "..." }` | Returns list of `{ text, href }` pairs |
| `tab.extractForms` | `{ "tab_id": "..." }` | Returns interactive form schemas and inputs |
| `tab.extractDom` | `{ "selector": "...", "tab_id": "..." }` | Returns raw HTML of page or CSS selector |
| `tab.click` | `{ "target": "selector_or_text", "tab_id": "..." }` | Simulates element click with auto-navigation |
| `tab.type` | `{ "selector": "...", "text": "...", "tab_id": "..." }` | Simulates keyboard text input |
| `tab.evaluateJs` | `{ "code": "...", "tab_id": "..." }` | Evaluates JavaScript expression in isolated runtime |
| `tab.setProfile` | `{ "profile": "SafariMac", "tab_id": "..." }` | Updates active device fingerprint |
| `engine.createTab` | `{ "profile": "ChromeWindows" }` | Spawns a new isolated tab, returns `tab_id` |
| `engine.closeTab` | `{ "tab_id": "tab_1" }` | Closes and cleans up a tab instance |
| `engine.listTabs` | `{}` | Lists all active tabs and their profiles |
| `shutdown` | `{}` | Gracefully terminates the engine process |

---

## 📜 License

Licensed under either of:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or [opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or [apache.org/licenses/LICENSE-2.0](https://www.apache.org/licenses/LICENSE-2.0))

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.
