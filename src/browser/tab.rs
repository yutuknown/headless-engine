use crate::browser::builder::BrowserBuilder;
use crate::dom::{DomTree, FormInfo, InteractiveElement, LinkInfo, PageObservation, SearchResults};
use crate::js::context::JsRuntime;
use crate::network::client::{FetchResult, NetworkClient};
use crate::network::fingerprint::DeviceProfile;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NavigationReport {
    pub status: u16,
    pub requested_url: String,
    pub final_url: String,
    pub page_title: String,
    pub is_captcha_detected: bool,
    pub html_bytes: usize,
}

pub struct BrowserTab {
    network: NetworkClient,
    dom: Option<DomTree>,
    js: JsRuntime,
    pub current_url: Option<String>,
}

impl BrowserTab {
    pub fn new() -> Result<Self> {
        Self::with_profile(DeviceProfile::ChromeWindows)
    }

    pub fn builder() -> BrowserBuilder {
        BrowserBuilder::new()
    }

    pub fn with_profile(profile: DeviceProfile) -> Result<Self> {
        let network = NetworkClient::with_profile(profile)?;
        Self::from_network(network)
    }

    pub fn from_network(network: NetworkClient) -> Result<Self> {
        let js = JsRuntime::with_fingerprint(&network.fingerprint)?;
        Ok(Self {
            network,
            dom: None,
            js,
            current_url: None,
        })
    }

    pub fn profile(&self) -> DeviceProfile {
        self.network.profile
    }

    pub fn set_profile(&mut self, profile: DeviceProfile) -> Result<()> {
        self.network.set_profile(profile)?;
        self.js = JsRuntime::with_fingerprint(&self.network.fingerprint)?;
        Ok(())
    }

    pub async fn navigate(&mut self, url: &str) -> Result<NavigationReport> {
        let fetch_result: FetchResult = self.network.fetch(url).await?;
        let dom = DomTree::parse(&fetch_result.html)?;

        let search_results = dom.parse_google_search_results();
        let page_title = search_results.page_title.clone();
        let html_bytes = fetch_result.html.len();

        let _ = self.js.update_page_state(&fetch_result.final_url, &page_title);

        self.dom = Some(dom);
        self.current_url = Some(fetch_result.final_url.clone());

        Ok(NavigationReport {
            status: fetch_result.status,
            requested_url: url.to_string(),
            final_url: fetch_result.final_url,
            page_title,
            is_captcha_detected: fetch_result.is_captcha_detected,
            html_bytes,
        })
    }

    pub fn observe(&self) -> Option<PageObservation> {
        let dom = self.dom.as_ref()?;
        let url = self.current_url.clone().unwrap_or_default();
        let results = dom.parse_google_search_results();
        let elements = dom.extract_interactive_elements(Some(&url));

        let mut tree_lines = Vec::new();
        for el in &elements {
            tree_lines.push(el.to_agent_string());
        }

        let agent_tree_text = tree_lines.join("\n");
        let content_summary_markdown = dom.extract_markdown(None, Some(&url));

        Some(PageObservation {
            url,
            title: results.page_title,
            is_captcha_detected: results.is_captcha_detected,
            interactive_elements: elements,
            agent_tree_text,
            content_summary_markdown,
        })
    }

    pub fn evaluate_js(&mut self, code: &str) -> Result<String> {
        self.js.evaluate(code)
    }

    pub fn extract_dom(&self, selector: Option<&str>) -> Option<String> {
        self.dom.as_ref().and_then(|d| d.extract(selector))
    }

    pub fn extract_markdown(&self, selector: Option<&str>) -> Option<String> {
        self.dom.as_ref().map(|d| d.extract_markdown(selector, self.current_url.as_deref()))
    }

    pub fn extract_interactive_elements(&self) -> Vec<InteractiveElement> {
        self.dom.as_ref().map(|d| d.extract_interactive_elements(self.current_url.as_deref())).unwrap_or_default()
    }

    pub fn extract_links(&self) -> Vec<LinkInfo> {
        self.dom.as_ref().map(|d| d.extract_links(self.current_url.as_deref())).unwrap_or_default()
    }

    pub fn extract_forms(&self) -> Vec<FormInfo> {
        self.dom.as_ref().map(|d| d.extract_forms()).unwrap_or_default()
    }

    pub fn extract_search_results(&self) -> Option<SearchResults> {
        self.dom.as_ref().map(|d| d.parse_google_search_results())
    }

    pub async fn screenshot_async(&self) -> Option<crate::dom::ScreenshotResult> {
        let dom = self.dom.as_ref()?;
        let url = self.current_url.as_deref().unwrap_or("about:blank");
        let results = dom.parse_google_search_results();
        Some(dom.screenshot_async(url, &results.page_title, self.current_url.as_deref()).await)
    }

    pub fn screenshot(&self) -> Option<crate::dom::ScreenshotResult> {
        let dom = self.dom.as_ref()?;
        let url = self.current_url.as_deref().unwrap_or("about:blank");
        let results = dom.parse_google_search_results();
        Some(dom.screenshot(url, &results.page_title, self.current_url.as_deref()))
    }

    pub fn screenshot_svg(&self) -> Option<String> {
        self.screenshot().map(|s| s.svg)
    }

    pub fn screenshot_layout(&self) -> Option<String> {
        self.screenshot().map(|s| s.layout_wireframe)
    }

    pub async fn act_click(&mut self, target: &str) -> Result<Option<NavigationReport>> {
        // Check if target is a numerical index
        if let Ok(idx) = target.parse::<usize>() {
            let elements = self.extract_interactive_elements();
            if let Some(el) = elements.iter().find(|e| e.index == idx) {
                if !el.href.is_empty() {
                    let report = self.navigate(&el.href).await?;
                    return Ok(Some(report));
                }
                return self.click(&el.selector).await;
            }
        }

        self.click(target).await
    }

    pub async fn act_type(&mut self, target: &str, text: &str) -> Result<String> {
        // Check if target is a numerical index
        if let Ok(idx) = target.parse::<usize>() {
            let elements = self.extract_interactive_elements();
            if let Some(el) = elements.iter().find(|e| e.index == idx) {
                return self.type_text(&el.selector, text);
            }
        }

        self.type_text(target, text)
    }

    pub async fn click(&mut self, selector_or_text: &str) -> Result<Option<NavigationReport>> {
        let links = self.extract_links();

        // 1. Check if matches href or anchor text
        if let Some(link) = links.iter().find(|l| {
            l.text.eq_ignore_ascii_case(selector_or_text)
                || l.href.contains(selector_or_text)
                || l.text.to_lowercase().contains(&selector_or_text.to_lowercase())
        }) {
            let report = self.navigate(&link.href).await?;
            return Ok(Some(report));
        }

        // 2. Try evaluating JS click
        let js_code = format!(
            "var el = document.querySelector('{}'); if (el) {{ el.click(); true; }} else {{ false; }}",
            selector_or_text
        );
        let _ = self.evaluate_js(&js_code);

        Ok(None)
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<String> {
        let js_code = format!(
            "var el = document.querySelector('{}'); if (el) {{ el.value = '{}'; 'updated'; }} else {{ 'not_found'; }}",
            selector, text.replace('\'', "\\'")
        );
        self.evaluate_js(&js_code).context("Failed to evaluate type action")
    }
}
