use crate::browser::builder::BrowserBuilder;
use crate::dom::{DomTree, FormInfo, InteractiveElement, LinkInfo, PageObservation, SearchResults};
use crate::js::context::JsRuntime;
use crate::network::client::{FetchResult, NetworkClient};
use crate::network::fingerprint::DeviceProfile;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::google::{GoogleEndpoints, GoogleParser, GoogleSearchResult, GenericGoogleResult, GoogleAutocompleteResult};#[derive(Debug, Serialize, Deserialize)]
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

        let _ = self
            .js
            .update_page_state(&fetch_result.final_url, &page_title);

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

    /// Default Google Search with automated query encoding and mode routing
    pub async fn search(&mut self, query: &str) -> Result<NavigationReport> {
        self.search_google(query, None).await
    }

    /// Search Google with specific query modes (e.g. "ai" for udm=50, "web" for udm=14, "images" for udm=2, "news")
    pub async fn search_google(&mut self, query: &str, mode: Option<&str>) -> Result<NavigationReport> {
        let encoded: String = query
            .chars()
            .map(|c| match c {
                ' ' => "+".to_string(),
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect();

        let url = match mode {
            Some("ai") | Some("udm50") => format!("https://www.google.com/search?q={}&udm=50", encoded),
            Some("web") | Some("udm14") => format!("https://www.google.com/search?q={}&udm=14", encoded),
            Some("images") | Some("udm2") => format!("https://www.google.com/search?q={}&udm=2", encoded),
            Some("news") => format!("https://www.google.com/search?q={}&tbm=nws", encoded),
            _ => format!("https://www.google.com/search?q={}", encoded),
        };
        self.navigate(&url).await
    }

    pub async fn google_search(&mut self, query: &str) -> Result<GoogleSearchResult> {
        let url = GoogleEndpoints::search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_search_results(&html, &nav.final_url))
    }

    pub async fn google_web_search(&mut self, query: &str) -> Result<GoogleSearchResult> {
        let url = GoogleEndpoints::web_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_search_results(&html, &nav.final_url))
    }

    pub async fn google_image_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::image_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_video_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::video_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_short_video_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::short_video_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_news_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::news_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_forum_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::forum_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_shopping_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::shopping_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_product_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::product_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_books_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::books_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_autocomplete(&mut self, query: &str) -> Result<GoogleAutocompleteResult> {
        let url = GoogleEndpoints::autocomplete(query);
        let fetch_result = self.network.fetch(&url).await?;
        Ok(GoogleParser::parse_autocomplete(&fetch_result.html))
    }

    pub async fn google_ai_overview(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::ai_overview(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_ai_mode(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::ai_mode(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_scholar_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::scholar_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_patents_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::patents_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_maps_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::maps_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_finance_quote(&mut self, ticker: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::finance_quote(ticker);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_trends_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::trends_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_flights_search(&mut self, origin: &str, dest: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::flights_search(origin, dest);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_hotels_search(&mut self, location: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::hotels_search(location);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_travel_explore(&mut self, destination: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::travel_explore(destination);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn youtube_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::youtube_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn youtube_shorts_search(&mut self, query: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::youtube_shorts_search(query);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn youtube_video(&mut self, video_id: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::youtube_video(video_id);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn youtube_channel(&mut self, channel: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::youtube_channel(channel);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn youtube_playlist(&mut self, playlist_id: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::youtube_playlist(playlist_id);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_lens_visual_matches(&mut self, image_url: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::lens_visual_matches(image_url);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_lens_exact_matches(&mut self, image_url: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::lens_exact_matches(image_url);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_lens_products(&mut self, image_url: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::lens_products(image_url);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub async fn google_lens_about_image(&mut self, image_url: &str) -> Result<GenericGoogleResult> {
        let url = GoogleEndpoints::lens_about_image(image_url);
        let nav = self.navigate(&url).await?;
        let html = self.dom.as_ref().map(|d| d.raw_content.clone()).unwrap_or_default();
        Ok(GoogleParser::parse_generic(&html, &nav.final_url))
    }

    pub fn google_capabilities(&self) -> Vec<&'static str> {
        vec![
            "google_search", "google_web_search", "google_image_search", "google_video_search",
            "google_short_video_search", "google_news_search", "google_forum_search", "google_shopping_search",
            "google_product_search", "google_books_search", "google_autocomplete", "google_ai_overview",
            "google_ai_mode", "google_scholar_search", "google_patents_search", "google_maps_search",
            "google_finance_quote", "google_trends_search", "google_flights_search", "google_hotels_search",
            "google_travel_explore", "youtube_search", "youtube_shorts_search", "youtube_video",
            "youtube_channel", "youtube_playlist", "google_lens_visual_matches", "google_lens_exact_matches",
            "google_lens_products", "google_lens_about_image", "google_capabilities"
        ]
    }

    pub fn set_content(&mut self, html: &str, url: Option<&str>) -> Result<NavigationReport> {
        let dom = DomTree::parse(html)?;
        let search_results = dom.parse_google_search_results();
        let page_title = search_results.page_title.clone();
        let final_url = url.unwrap_or("about:blank").to_string();
        let html_bytes = html.len();

        let _ = self.js.update_page_state(&final_url, &page_title);

        self.dom = Some(dom);
        self.current_url = Some(final_url.clone());

        Ok(NavigationReport {
            status: 200,
            requested_url: final_url.clone(),
            final_url,
            page_title,
            is_captcha_detected: search_results.is_captcha_detected,
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
        self.dom
            .as_ref()
            .map(|d| d.extract_markdown(selector, self.current_url.as_deref()))
    }

    pub fn extract_interactive_elements(&self) -> Vec<InteractiveElement> {
        self.dom
            .as_ref()
            .map(|d| d.extract_interactive_elements(self.current_url.as_deref()))
            .unwrap_or_default()
    }

    pub fn extract_links(&self) -> Vec<LinkInfo> {
        self.dom
            .as_ref()
            .map(|d| d.extract_links(self.current_url.as_deref()))
            .unwrap_or_default()
    }

    pub fn extract_forms(&self) -> Vec<FormInfo> {
        self.dom
            .as_ref()
            .map(|d| d.extract_forms())
            .unwrap_or_default()
    }

    pub fn extract_search_results(&self) -> Option<SearchResults> {
        self.dom.as_ref().map(|d| d.parse_google_search_results())
    }

    pub async fn screenshot_async(&self) -> Option<crate::dom::ScreenshotResult> {
        let dom = self.dom.as_ref()?;
        let url = self.current_url.as_deref().unwrap_or("about:blank");
        let results = dom.parse_google_search_results();
        Some(
            dom.screenshot_async(url, &results.page_title, self.current_url.as_deref())
                .await,
        )
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
                || l.text
                    .to_lowercase()
                    .contains(&selector_or_text.to_lowercase())
        }) {
            let report = self.navigate(&link.href).await?;
            return Ok(Some(report));
        }

        // 2. Try evaluating JS click
        let sel_json = serde_json::to_string(selector_or_text).unwrap_or_else(|_| format!("\"{}\"", selector_or_text));
        let js_code = format!(
            "var el = document.querySelector({}); if (el) {{ try {{ el.click(); }} catch(e){{}} true; }} else {{ false; }}",
            sel_json
        );
        let _ = self.evaluate_js(&js_code);

        Ok(None)
    }

    pub fn type_text(&mut self, selector: &str, text: &str) -> Result<String> {
        let sel_json = serde_json::to_string(selector).unwrap_or_else(|_| format!("\"{}\"", selector));
        let text_json = serde_json::to_string(text).unwrap_or_else(|_| format!("\"{}\"", text));
        let js_code = format!(
            r#"var el = document.querySelector({sel});
            if (el) {{
                if ('value' in el) {{
                    el.value = {txt};
                }} else {{
                    el.innerText = {txt};
                    el.textContent = {txt};
                }}
                try {{
                    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                }} catch(e) {{}}
                'updated';
            }} else {{
                'not_found';
            }}"#,
            sel = sel_json,
            txt = text_json
        );
        self.evaluate_js(&js_code)
            .context("Failed to evaluate type action")
    }
}
