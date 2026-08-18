use crate::dom::interactive::InteractiveElement;
use crate::dom::SearchResults;
use anyhow::Result;
use base64::Engine;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub width: u32,
    pub height: u32,
    pub svg: String,
    pub layout_wireframe: String,
    pub element_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub png_bytes: Vec<u8>,
    pub png_base64: String,
}

pub struct RealBrowserScreenshot;

struct TempFileCleanup(PathBuf);
impl Drop for TempFileCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

impl RealBrowserScreenshot {
    pub fn find_browser_binary() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let candidates = [
                r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
                r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
                r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            ];
            for path_str in &candidates {
                let p = Path::new(path_str);
                if p.exists() {
                    return Some(p.to_path_buf());
                }
            }
            if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
                let chrome_local = Path::new(&local_app_data).join(r"Google\Chrome\Application\chrome.exe");
                if chrome_local.exists() {
                    return Some(chrome_local);
                }
                let edge_local = Path::new(&local_app_data).join(r"Microsoft\Edge\Application\msedge.exe");
                if edge_local.exists() {
                    return Some(edge_local);
                }
            }
            // Dynamic fallback
            if let Ok(output) = std::process::Command::new("where").arg("chrome").output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").trim().to_string();
                    if !path_str.is_empty() { return Some(PathBuf::from(path_str)); }
                }
            }
            if let Ok(output) = std::process::Command::new("where").arg("msedge").output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("").trim().to_string();
                    if !path_str.is_empty() { return Some(PathBuf::from(path_str)); }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let candidates = [
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
            ];
            for path_str in &candidates {
                let p = Path::new(path_str);
                if p.exists() {
                    return Some(p.to_path_buf());
                }
            }
            // Dynamic fallback
            if let Ok(output) = std::process::Command::new("which").arg("google-chrome").output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path_str.is_empty() { return Some(PathBuf::from(path_str)); }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let candidates = ["google-chrome", "google-chrome-stable", "chromium", "chromium-browser"];
            for bin in &candidates {
                if let Ok(output) = Command::new("which").arg(bin).output() {
                    if output.status.success() {
                        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path_str.is_empty() {
                            return Some(PathBuf::from(path_str));
                        }
                    }
                }
            }
        }

        None
    }

    fn generate_temp_png_path() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let pid = std::process::id();
        let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);
        temp_dir.join(format!("headless_shot_{}_{}_{}.png", pid, time, count))
    }

    fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://")
    }

    pub async fn capture_real_screenshot_async(
        url: &str,
        html_str: &str,
        width: u32,
        height: u32,
    ) -> Option<ScreenshotResult> {
        if !Self::is_valid_url(url) {
            return None;
        }

        let browser_bin = Self::find_browser_binary()?;
        
        let temp_png = Self::generate_temp_png_path();
        let mut temp_html = temp_png.clone();
        temp_html.set_extension("html");
        
        let mut injected_html = html_str.to_string();
        let base_tag = format!("<base href=\"{}\">", url);
        if let Some(idx) = injected_html.find("<head>") {
            injected_html.insert_str(idx + 6, &base_tag);
        } else if let Some(idx) = injected_html.find("<head ") {
            if let Some(close_idx) = injected_html[idx..].find('>') {
                injected_html.insert_str(idx + close_idx + 1, &base_tag);
            }
        } else {
            injected_html.insert_str(0, &base_tag);
        }
        
        let _ = std::fs::write(&temp_html, injected_html);
        
        let _cleanup_png = TempFileCleanup(temp_png.clone());
        let _cleanup_html = TempFileCleanup(temp_html.clone());
        
        let temp_png_str = temp_png.to_string_lossy().to_string();
        let temp_html_str = format!("file:///{}", temp_html.to_string_lossy().to_string().replace('\\', "/"));
        
        let screenshot_arg = format!("--screenshot={}", temp_png_str);
        let window_size_arg = format!("--window-size={},{}", width, height);

        let mut cmd = tokio::process::Command::new(&browser_bin);
        cmd.arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--hide-scrollbars")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--virtual-time-budget=8000")
            .arg(&window_size_arg)
            .arg(&screenshot_arg)
            .arg(&temp_html_str);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        if let Ok(child) = cmd.spawn() {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(12), child.wait_with_output()).await;
        }

        if temp_png.exists() {
            if let Ok(png_bytes) = std::fs::read(&temp_png) {
                let png_base64 = format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(&png_bytes)
                );
                return Some(ScreenshotResult {
                    width,
                    height,
                    svg: String::new(),
                    layout_wireframe: String::new(),
                    element_count: 1,
                    png_bytes,
                    png_base64,
                });
            }
        }
        None
    }

    pub fn capture_real_screenshot_sync(
        url: &str,
        html_str: &str,
        width: u32,
        height: u32,
    ) -> Option<ScreenshotResult> {
        if !Self::is_valid_url(url) {
            return None;
        }

        let browser_bin = Self::find_browser_binary()?;
        
        let temp_png = Self::generate_temp_png_path();
        let mut temp_html = temp_png.clone();
        temp_html.set_extension("html");
        
        let mut injected_html = html_str.to_string();
        let base_tag = format!("<base href=\"{}\">", url);
        if let Some(idx) = injected_html.find("<head>") {
            injected_html.insert_str(idx + 6, &base_tag);
        } else if let Some(idx) = injected_html.find("<head ") {
            if let Some(close_idx) = injected_html[idx..].find('>') {
                injected_html.insert_str(idx + close_idx + 1, &base_tag);
            }
        } else {
            injected_html.insert_str(0, &base_tag);
        }
        
        let _ = std::fs::write(&temp_html, injected_html);
        
        let _cleanup_png = TempFileCleanup(temp_png.clone());
        let _cleanup_html = TempFileCleanup(temp_html.clone());
        
        let temp_png_str = temp_png.to_string_lossy().to_string();
        let temp_html_str = format!("file:///{}", temp_html.to_string_lossy().to_string().replace('\\', "/"));
        
        let screenshot_arg = format!("--screenshot={}", temp_png_str);
        let window_size_arg = format!("--window-size={},{}", width, height);

        let mut cmd = std::process::Command::new(&browser_bin);
        cmd.arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--no-sandbox")
            .arg("--hide-scrollbars")
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--virtual-time-budget=8000")
            .arg(&window_size_arg)
            .arg(&screenshot_arg)
            .arg(&temp_html_str);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        if let Ok(mut child) = cmd.spawn() {
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(12);
            loop {
                if let Ok(Some(_)) = child.try_wait() {
                    break;
                }
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        if temp_png.exists() {
            if let Ok(png_bytes) = std::fs::read(&temp_png) {
                let png_base64 = format!(
                    "data:image/png;base64,{}",
                    base64::engine::general_purpose::STANDARD.encode(&png_bytes)
                );
                return Some(ScreenshotResult {
                    width,
                    height,
                    svg: String::new(),
                    layout_wireframe: String::new(),
                    element_count: 1,
                    png_bytes,
                    png_base64,
                });
            }
        }
        None
    }
}

pub struct PageRenderer;

impl PageRenderer {
    pub async fn render_async(
        url: &str,
        title: &str,
        html_str: &str,
        interactive: &[InteractiveElement],
        _search_results: Option<&SearchResults>,
    ) -> ScreenshotResult {
        let width = 1280;
        let height = 720;

        // Try one-shot ultra-light real browser screenshot first
        if let Some(real_shot) = RealBrowserScreenshot::capture_real_screenshot_async(url, html_str, width, height).await {
            return real_shot;
        }

        // Fallback to pure-Rust layout & rasterization engine
        crate::render::HtmlRenderer::render_html_to_screenshot(url, title, html_str, interactive, width, height)
            .await
            .unwrap_or_else(|_| Self::render_general_page(url, title, html_str, interactive, width, height))
    }

    pub fn render(
        url: &str,
        title: &str,
        html_str: &str,
        interactive: &[InteractiveElement],
        _search_results: Option<&SearchResults>,
    ) -> ScreenshotResult {
        let width = 1280;
        let height = 720;

        if let Some(real_shot) = RealBrowserScreenshot::capture_real_screenshot_sync(url, html_str, width, height) {
            return real_shot;
        }

        Self::render_general_page(url, title, html_str, interactive, width, height)
    }

    fn render_general_page(
        url: &str,
        title: &str,
        html_str: &str,
        interactive: &[InteractiveElement],
        width: u32,
        height: u32,
    ) -> ScreenshotResult {
        let mut y_offset = 120;
        let document = Html::parse_document(html_str);

        let mut visual_blocks = Vec::new();
        if let Ok(sel) = Selector::parse("h1, h2, h3, p, button, a[href], input, li") {
            for el in document.select(&sel) {
                let tag = el.value().name();
                let text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if text.is_empty() || text.len() < 2 {
                    continue;
                }

                let linked_index = interactive.iter().find(|i| i.text == text).map(|i| i.index);
                visual_blocks.push((tag.to_string(), text, linked_index));
                if visual_blocks.len() >= 50 {
                    break;
                }
            }
        }

        let title_escaped = Self::xml_escape(&title.chars().take(30).collect::<String>());
        let url_escaped = Self::xml_escape(url);

        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\" style=\"background:#0f172a; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;\">\n\
  <rect width=\"{width}\" height=\"80\" fill=\"#1e293b\" />\n\
  <circle cx=\"25\" cy=\"40\" r=\"7\" fill=\"#ef4444\" />\n\
  <circle cx=\"45\" cy=\"40\" r=\"7\" fill=\"#f59e0b\" />\n\
  <circle cx=\"65\" cy=\"40\" r=\"7\" fill=\"#10b981\" />\n\
  <rect x=\"90\" y=\"24\" width=\"900\" height=\"32\" rx=\"6\" fill=\"#0f172a\" stroke=\"#334155\" stroke-width=\"1\" />\n\
  <text x=\"105\" y=\"45\" fill=\"#94a3b8\" font-size=\"13\">&#128274; {url_escaped}</text>\n\
  <text x=\"1010\" y=\"45\" fill=\"#e2e8f0\" font-size=\"13\" font-weight=\"600\">{title_escaped}</text>\n\
  <g transform=\"translate(60, 100)\">\n"
        );

        let mut wireframe_lines = Vec::new();
        wireframe_lines.push("╔══════════════════════════════════════════════════════════════════════════════╗".to_string());
        wireframe_lines.push(format!("║ URL: {:<72} ║", url));
        wireframe_lines.push(format!("║ TITLE: {:<70} ║", title));
        wireframe_lines.push("╠══════════════════════════════════════════════════════════════════════════════╣".to_string());

        for (tag, text, index_opt) in &visual_blocks {
            let truncated_text: String = text.chars().take(80).collect();
            let escaped_text = Self::xml_escape(&truncated_text);

            match tag.as_str() {
                "h1" => {
                    svg.push_str(&format!(
                        "    <text x=\"0\" y=\"{}\" fill=\"#f8fafc\" font-size=\"24\" font-weight=\"bold\">{}</text>\n",
                        y_offset, escaped_text
                    ));
                    wireframe_lines.push(format!("║ # {:<74} ║", truncated_text));
                    y_offset += 40;
                }
                "h2" => {
                    svg.push_str(&format!(
                        "    <text x=\"0\" y=\"{}\" fill=\"#38bdf8\" font-size=\"18\" font-weight=\"bold\">{}</text>\n",
                        y_offset, escaped_text
                    ));
                    wireframe_lines.push(format!("║ ## {:<73} ║", truncated_text));
                    y_offset += 32;
                }
                _ => {
                    let badge = index_opt.map(|i| format!("[{}] ", i)).unwrap_or_default();
                    let color = if index_opt.is_some() { "#60a5fa" } else { "#cbd5e1" };
                    svg.push_str(&format!(
                        "    <text x=\"0\" y=\"{}\" fill=\"{}\" font-size=\"13\">{}{}</text>\n",
                        y_offset, color, badge, escaped_text
                    ));
                    if index_opt.is_some() {
                        wireframe_lines.push(format!("║ [LINK] {}{:<67} ║", badge, truncated_text));
                    } else {
                        wireframe_lines.push(format!("║ {:<76} ║", truncated_text));
                    }
                    y_offset += 22;
                }
            }

            if y_offset > 1100 {
                break;
            }
        }

        svg.push_str("  </g>\n</svg>");
        wireframe_lines.push("╚══════════════════════════════════════════════════════════════════════════════╝".to_string());

        let png_bytes = Self::render_png(&svg, width, height).unwrap_or_default();
        let png_base64 = if !png_bytes.is_empty() {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&png_bytes)
            )
        } else {
            String::new()
        };

        ScreenshotResult {
            width,
            height,
            svg,
            layout_wireframe: wireframe_lines.join("\n"),
            element_count: visual_blocks.len(),
            png_bytes,
            png_base64,
        }
    }

    pub fn render_png(svg_str: &str, width: u32, height: u32) -> Result<Vec<u8>> {
        let opt = resvg::usvg::Options {
            font_family: "sans-serif".to_string(),
            ..Default::default()
        };
        let tree = resvg::usvg::Tree::from_str(svg_str, &opt)?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| anyhow::anyhow!("Failed to allocate raster pixmap"))?;

        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
        let png_data = pixmap.encode_png()?;
        Ok(png_data)
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
