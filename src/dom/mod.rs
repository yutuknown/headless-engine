pub mod interactive;
pub mod markdown;
pub mod screenshot;

pub use interactive::{InteractiveElement, InteractiveParser, PageObservation};
pub use screenshot::{PageRenderer, ScreenshotResult};
use anyhow::Result;
use markdown::HtmlToMarkdown;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganicResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsResult {
    pub headline: String,
    pub source: String,
    pub time_ago: String,
    pub link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResult {
    pub title: String,
    pub video_id: String,
    pub url: String,
    pub channel: String,
    pub duration: String,
    pub views: String,
    pub published_time: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResult {
    pub title: String,
    pub image_url: String,
    pub source_url: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOverview {
    pub summary: String,
    pub source_references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePanel {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub text: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInputInfo {
    pub name: String,
    pub input_type: String,
    pub value: String,
    pub placeholder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormInfo {
    pub action: String,
    pub method: String,
    pub inputs: Vec<FormInputInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub page_title: String,
    pub ai_overview: Option<AiOverview>,
    pub knowledge_panel: Option<KnowledgePanel>,
    pub image_results: Vec<ImageResult>,
    pub video_results: Vec<VideoResult>,
    pub news_results: Vec<NewsResult>,
    pub organic_results: Vec<OrganicResult>,
    pub related_questions: Vec<String>,
    pub is_captcha_detected: bool,
    pub total_results_found: usize,
}

pub struct DomTree {
    pub raw_content: String,
    document: Html,
}

impl DomTree {
    pub fn parse(content: &str) -> Result<Self> {
        let document = Html::parse_document(content);
        Ok(Self {
            raw_content: content.to_string(),
            document,
        })
    }

    pub fn extract(&self, selector_str: Option<&str>) -> Option<String> {
        if let Some(sel) = selector_str {
            if let Ok(selector) = Selector::parse(sel) {
                let matches: Vec<String> = self
                    .document
                    .select(&selector)
                    .map(|el| el.html())
                    .collect();
                if !matches.is_empty() {
                    return Some(matches.join("\n"));
                }
            }
        }
        Some(self.raw_content.clone())
    }

    pub fn extract_markdown(&self, selector_str: Option<&str>, base_url: Option<&str>) -> String {
        if let Some(sel) = selector_str {
            if let Ok(selector) = Selector::parse(sel) {
                let parts: Vec<String> = self
                    .document
                    .select(&selector)
                    .map(|el| HtmlToMarkdown::convert_element(&el, base_url))
                    .collect();
                if !parts.is_empty() {
                    return parts.join("\n\n---\n\n");
                }
            }
        }
        HtmlToMarkdown::convert(&self.raw_content, base_url)
    }

    pub fn extract_interactive_elements(&self, base_url: Option<&str>) -> Vec<InteractiveElement> {
        InteractiveParser::parse(&self.raw_content, base_url)
    }

    pub async fn screenshot_async(&self, url: &str, title: &str, base_url: Option<&str>) -> ScreenshotResult {
        let interactive = self.extract_interactive_elements(base_url);
        let search_results = self.parse_google_search_results();
        PageRenderer::render_async(url, title, &self.raw_content, &interactive, Some(&search_results)).await
    }

    pub fn screenshot(&self, url: &str, title: &str, base_url: Option<&str>) -> ScreenshotResult {
        let interactive = self.extract_interactive_elements(base_url);
        let search_results = self.parse_google_search_results();
        PageRenderer::render(url, title, &self.raw_content, &interactive, Some(&search_results))
    }

    pub fn extract_links(&self, base_url: Option<&str>) -> Vec<LinkInfo> {
        let mut links = Vec::new();
        if let Ok(a_sel) = Selector::parse("a[href]") {
            for a in self.document.select(&a_sel) {
                let raw_href = a.value().attr("href").unwrap_or("");
                let text = a.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !raw_href.is_empty() && !raw_href.starts_with("javascript:") {
                    let full_url = if let Some(base) = base_url {
                        if raw_href.starts_with("http://") || raw_href.starts_with("https://") {
                            raw_href.to_string()
                        } else if raw_href.starts_with("//") {
                            format!("https:{}", raw_href)
                        } else if raw_href.starts_with('/') {
                            if let Some(idx) = base.find("://") {
                                let after = &base[idx + 3..];
                                let host = after.split('/').next().unwrap_or(after);
                                let scheme = &base[..idx + 3];
                                format!("{}{}{}", scheme, host, raw_href)
                            } else {
                                raw_href.to_string()
                            }
                        } else {
                            format!("{}/{}", base.trim_end_matches('/'), raw_href)
                        }
                    } else {
                        raw_href.to_string()
                    };

                    if !links.iter().any(|l: &LinkInfo| l.href == full_url) {
                        links.push(LinkInfo {
                            text: if text.is_empty() { full_url.clone() } else { text },
                            href: full_url,
                        });
                    }
                }
            }
        }
        links
    }

    pub fn extract_forms(&self) -> Vec<FormInfo> {
        let mut forms = Vec::new();
        if let Ok(form_sel) = Selector::parse("form") {
            let input_sel = Selector::parse("input, textarea, select").ok();
            for form in self.document.select(&form_sel) {
                let action = form.value().attr("action").unwrap_or("").to_string();
                let method = form.value().attr("method").unwrap_or("GET").to_uppercase();

                let mut inputs = Vec::new();
                if let Some(ref in_sel) = input_sel {
                    for inp in form.select(in_sel) {
                        let name = inp.value().attr("name").unwrap_or("").to_string();
                        let input_type = inp.value().attr("type").unwrap_or("text").to_string();
                        let value = inp.value().attr("value").unwrap_or("").to_string();
                        let placeholder = inp.value().attr("placeholder").unwrap_or("").to_string();

                        inputs.push(FormInputInfo {
                            name,
                            input_type,
                            value,
                            placeholder,
                        });
                    }
                }

                forms.push(FormInfo {
                    action,
                    method,
                    inputs,
                });
            }
        }
        forms
    }

    pub fn parse_google_search_results(&self) -> SearchResults {
        let title_selector = Selector::parse("title").ok();
        let page_title = title_selector
            .and_then(|sel| self.document.select(&sel).next())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_else(|| "Search Results".to_string());

        let is_captcha_detected = page_title.contains("Sorry")
            || page_title.contains("unusual traffic")
            || self.raw_content.contains("sorry/index?continue=")
            || self.raw_content.contains("id=\"captcha-form\"")
            || (self.raw_content.contains("challenges.cloudflare.com") && self.raw_content.contains("cf-turnstile-wrapper"))
            || self.raw_content.contains("hcaptcha-box");

        let mut organic_results = Vec::new();
        let mut news_results = Vec::new();
        let mut video_results = Vec::new();
        let mut image_results = Vec::new();
        let mut related_questions = Vec::new();
        let mut ai_overview: Option<AiOverview> = None;
        let mut knowledge_panel: Option<KnowledgePanel> = None;

        // 1. AI Overview Extraction (Google Search Generative Experience / Quick Answers)
        if let Ok(ai_sel) = Selector::parse("div[data-attrid='wa:/description'], div.YzSd6e, div.NFZabb, div.V3FYCf, div[aria-label*='AI Overview'], div.kno-rdesc") {
            if let Some(ai_el) = self.document.select(&ai_sel).next() {
                let summary = ai_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !summary.is_empty() && summary.len() > 20 {
                    let mut source_refs = Vec::new();
                    if let Ok(ref_sel) = Selector::parse("a[href]") {
                        for a in ai_el.select(&ref_sel) {
                            if let Some(href) = a.value().attr("href") {
                                if href.starts_with("http") && !source_refs.contains(&href.to_string()) {
                                    source_refs.push(href.to_string());
                                }
                            }
                        }
                    }
                    ai_overview = Some(AiOverview {
                        summary,
                        source_references: source_refs,
                    });
                }
            }
        }

        // 2. Knowledge Panel Extraction (Entities, Celebrities, Places, Organizations)
        if let Ok(kp_title_sel) = Selector::parse("div[data-attrid='title'], h2[data-attrid='title'], div.BNeawe.vvjwJb") {
            if let Some(kp_title_el) = self.document.select(&kp_title_sel).next() {
                let title = kp_title_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !title.is_empty() {
                    let subtitle = Selector::parse("div[data-attrid='subtitle'], div.BNeawe.UPmit")
                        .ok()
                        .and_then(|s| self.document.select(&s).next())
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_default();

                    let description = Selector::parse("div[data-attrid='description'], div.kno-rdesc")
                        .ok()
                        .and_then(|s| self.document.select(&s).next())
                        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_default();

                    let mut attributes = Vec::new();
                    if let Ok(attr_sel) = Selector::parse("div.rVusze, div[data-attrid]:not([data-attrid='title']):not([data-attrid='subtitle'])") {
                        for attr_el in self.document.select(&attr_sel) {
                            let text = attr_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                            if let Some(colon_idx) = text.find(':') {
                                let k = text[..colon_idx].trim().to_string();
                                let v = text[colon_idx + 1..].trim().to_string();
                                if !k.is_empty() && !v.is_empty() {
                                    attributes.push((k, v));
                                }
                            }
                        }
                    }

                    if !description.is_empty() || !attributes.is_empty() {
                        knowledge_panel = Some(KnowledgePanel {
                            title,
                            subtitle,
                            description,
                            attributes,
                        });
                    }
                }
            }
        }

        // 3. Google Images Mode Extraction (`udm=2` / `tbm=isch` / image galleries)
        if let Ok(img_box_sel) = Selector::parse("div[data-ri], div.isv-r, div.F0uyec, div.eA0Zlc, table.e2BEnf") {
            let img_sel = Selector::parse("img").ok();
            let a_sel = Selector::parse("a[href]").ok();

            for img_box in self.document.select(&img_box_sel) {
                let img_el = img_sel.as_ref().and_then(|s| img_box.select(s).next());
                let image_url = img_el
                    .and_then(|img| {
                        img.value().attr("src")
                            .or_else(|| img.value().attr("data-src"))
                            .or_else(|| img.value().attr("data-iurl"))
                    })
                    .unwrap_or_default()
                    .to_string();

                let a_el = a_sel.as_ref().and_then(|s| img_box.select(s).next());
                let source_url = a_el
                    .and_then(|a| a.value().attr("href"))
                    .unwrap_or_default()
                    .to_string();

                let title = img_box.text().collect::<Vec<_>>().join(" ").trim().to_string();
                let domain = if let Some(idx) = source_url.find("://") {
                    let after = &source_url[idx + 3..];
                    after.split('/').next().unwrap_or_default().to_string()
                } else {
                    String::new()
                };

                if !image_url.is_empty() && !image_results.iter().any(|i: &ImageResult| i.image_url == image_url) {
                    image_results.push(ImageResult {
                        title,
                        image_url,
                        source_url,
                        domain,
                    });
                }
            }
        }

        // 4. YouTube Search InitialData Extraction
        if self.raw_content.contains("ytInitialData") {
            if let Some(start_idx) = self.raw_content.find("var ytInitialData =")
                .or_else(|| self.raw_content.find("ytInitialData ="))
            {
                let rest = &self.raw_content[start_idx..];
                if let Some(brace_idx) = rest.find('{') {
                    let json_str = &rest[brace_idx..];
                    if let Some(semi_idx) = json_str.find(";</script>")
                        .or_else(|| json_str.find(";\n"))
                        .or_else(|| json_str.find(";var "))
                    {
                        let candidate = &json_str[..semi_idx];
                        if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(candidate) {
                            Self::extract_youtube_videos(&parsed_json, &mut video_results);
                        }
                    }
                }
            }

            for v in &video_results {
                let snippet = format!(
                    "Channel: {} | Duration: {} | Views: {} | Uploaded: {} - {}",
                    v.channel, v.duration, v.views, v.published_time, v.description
                );
                organic_results.push(OrganicResult {
                    title: v.title.clone(),
                    link: v.url.clone(),
                    snippet,
                });
            }
        }

        // 5. RSS / XML News Feed Extraction
        if self.raw_content.contains("<item>") || self.raw_content.contains("<entry>") {
            if let Ok(item_sel) = Selector::parse("item, entry") {
                let title_sel = Selector::parse("title").ok();
                let pubdate_sel = Selector::parse("pubDate, published, updated").ok();
                let desc_sel = Selector::parse("description, summary").ok();

                for item in self.document.select(&item_sel) {
                    let full_title = title_sel
                        .as_ref()
                        .and_then(|s| item.select(s).next())
                        .map(|t| t.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();

                    if full_title.is_empty() {
                        continue;
                    }

                    let (headline, source) = if let Some(idx) = full_title.rfind(" - ") {
                        (full_title[..idx].trim().to_string(), full_title[idx + 3..].trim().to_string())
                    } else {
                        (full_title.clone(), "Google News".to_string())
                    };

                    let time_ago = pubdate_sel
                        .as_ref()
                        .and_then(|s| item.select(s).next())
                        .map(|t| t.text().collect::<String>().trim().to_string())
                        .unwrap_or_default();

                    let desc_html = desc_sel
                        .as_ref()
                        .and_then(|s| item.select(s).next())
                        .map(|t| t.inner_html())
                        .unwrap_or_default();

                    let mut link = String::new();
                    if let Some(href_idx) = desc_html.find("href=\"") {
                        let after = &desc_html[href_idx + 6..];
                        if let Some(end_idx) = after.find('"') {
                            link = after[..end_idx].to_string();
                        }
                    }

                    let snippet = {
                        let desc_doc = Html::parse_fragment(&desc_html);
                        desc_doc.root_element().text().collect::<Vec<_>>().join(" ").trim().to_string()
                    };

                    news_results.push(NewsResult {
                        headline: headline.clone(),
                        source,
                        time_ago,
                        link: link.clone(),
                    });

                    organic_results.push(OrganicResult {
                        title: headline,
                        link,
                        snippet,
                    });
                }
            }
        }

        // 6. DuckDuckGo / Universal HTML SERP Extraction
        if let Ok(ddg_sel) = Selector::parse("div.result, div.web-result, div.results_links") {
            let title_sel = Selector::parse("a.result__url, h2.result__title a, a.result__a").ok();
            let snip_sel = Selector::parse("a.result__snippet, div.result__snippet").ok();

            for res_el in self.document.select(&ddg_sel) {
                let title_el = title_sel.as_ref().and_then(|s| res_el.select(s).next());
                let title = title_el
                    .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();
                let link = title_el
                    .and_then(|t| t.value().attr("href"))
                    .unwrap_or_default()
                    .to_string();
                let snippet = snip_sel
                    .as_ref()
                    .and_then(|s| res_el.select(s).next())
                    .map(|sn| sn.text().collect::<Vec<_>>().join(" ").trim().to_string())
                    .unwrap_or_default();

                if !title.is_empty() && !link.is_empty() && !organic_results.iter().any(|r| r.link == link) {
                    organic_results.push(OrganicResult {
                        title,
                        link,
                        snippet,
                    });
                }
            }
        }

        // 7. Google / Bing Standard HTML SERP Extraction (with Videos and Organic snippets)
        if let Ok(h3_selector) = Selector::parse("h3, h2, div[role='heading']") {
            let a_selector = Selector::parse("a[href]").ok();

            for h3_el in self.document.select(&h3_selector) {
                let title = h3_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if title.is_empty()
                    || title.eq_ignore_ascii_case("search results")
                    || title.eq_ignore_ascii_case("people also ask")
                    || title.len() < 3
                {
                    continue;
                }

                let mut found_link = String::new();
                let mut current = h3_el.parent();

                for _ in 0..6 {
                    if let Some(parent_node) = current {
                        if let Some(el_ref) = scraper::ElementRef::wrap(parent_node) {
                            if el_ref.value().name() == "a" {
                                if let Some(href) = el_ref.value().attr("href") {
                                    found_link = href.to_string();
                                    break;
                                }
                            }
                            if let Some(ref a_sel) = a_selector {
                                if let Some(a_el) = el_ref.select(a_sel).next() {
                                    if let Some(href) = a_el.value().attr("href") {
                                        found_link = href.to_string();
                                        break;
                                    }
                                }
                            }
                        }
                        current = parent_node.parent();
                    } else {
                        break;
                    }
                }

                let mut clean_url = found_link;
                if clean_url.starts_with("/url?q=") {
                    if let Some(end_idx) = clean_url.find("&sa=") {
                        clean_url = clean_url[7..end_idx].to_string();
                    } else {
                        clean_url = clean_url[7..].to_string();
                    }
                }

                if !clean_url.starts_with("http")
                    || clean_url.contains("google.com/")
                    || clean_url.contains("bing.com/")
                    || clean_url.contains("duckduckgo.com/")
                {
                    continue;
                }

                let mut snippet = String::new();
                if let Some(parent_node) = h3_el.parent().and_then(|p| p.parent()) {
                    if let Some(container_el) = scraper::ElementRef::wrap(parent_node) {
                        let full_text = container_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                        if full_text.starts_with(&title) {
                            snippet = full_text[title.len()..].trim().to_string();
                        } else {
                            snippet = full_text;
                        }
                    }
                }

                // Check if this is a video result on Google Videos
                if clean_url.contains("youtube.com/watch") || clean_url.contains("vimeo.com") {
                    let video_id = if let Some(idx) = clean_url.find("v=") {
                        clean_url[idx + 2..].split('&').next().unwrap_or_default().to_string()
                    } else {
                        String::new()
                    };

                    if !video_results.iter().any(|v| v.url == clean_url) {
                        video_results.push(VideoResult {
                            title: title.clone(),
                            video_id,
                            url: clean_url.clone(),
                            channel: "Web Video".to_string(),
                            duration: String::new(),
                            views: String::new(),
                            published_time: String::new(),
                            description: snippet.clone(),
                        });
                    }
                }

                let lower_snippet = snippet.to_lowercase();
                if lower_snippet.contains("hours ago")
                    || lower_snippet.contains("days ago")
                    || lower_snippet.contains("mins ago")
                {
                    let parts: Vec<&str> = snippet.split('·').collect();
                    let source = if parts.len() > 1 {
                        parts[0].trim().to_string()
                    } else {
                        "Web".to_string()
                    };
                    let time_ago = if parts.len() > 1 {
                        parts[1].trim().to_string()
                    } else {
                        snippet.clone()
                    };

                    if !news_results.iter().any(|n| n.headline == title || n.link == clean_url) {
                        news_results.push(NewsResult {
                            headline: title.clone(),
                            source,
                            time_ago,
                            link: clean_url.clone(),
                        });
                    }
                }

                if !organic_results.iter().any(|r| r.link == clean_url) {
                    organic_results.push(OrganicResult {
                        title,
                        link: clean_url,
                        snippet,
                    });
                }
            }
        }

        // 8. People Also Ask / Related Questions
        if let Ok(q_selector) = Selector::parse("div.cb7Db, div[data-q], div.related-question-pair, div.CSkcDe") {
            for q_el in self.document.select(&q_selector) {
                let q_text = q_el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !q_text.is_empty() && q_text.len() > 5 && !related_questions.contains(&q_text) {
                    related_questions.push(q_text);
                }
            }
        }

        let total_results_found = organic_results.len() + news_results.len() + video_results.len() + image_results.len();

        SearchResults {
            page_title,
            ai_overview,
            knowledge_panel,
            image_results,
            video_results,
            news_results,
            organic_results,
            related_questions,
            is_captcha_detected,
            total_results_found,
        }
    }

    fn extract_youtube_videos(val: &serde_json::Value, list: &mut Vec<VideoResult>) {
        match val {
            serde_json::Value::Object(map) => {
                if let Some(vr) = map.get("videoRenderer") {
                    let video_id = vr
                        .get("videoId")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();

                    if !video_id.is_empty() {
                        let title = vr
                            .get("title")
                            .and_then(|t| t.get("runs"))
                            .and_then(|r| r.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|item| item.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or_default()
                            .to_string();

                        let channel = vr
                            .get("ownerText")
                            .and_then(|o| o.get("runs"))
                            .and_then(|r| r.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|item| item.get("text"))
                            .and_then(|t| t.as_str())
                            .unwrap_or_default()
                            .to_string();

                        let duration = vr
                            .get("lengthText")
                            .and_then(|l| l.get("simpleText"))
                            .and_then(|t| t.as_str())
                            .unwrap_or_default()
                            .to_string();

                        let views = vr
                            .get("viewCountText")
                            .and_then(|v| v.get("simpleText"))
                            .and_then(|t| t.as_str())
                            .unwrap_or_default()
                            .to_string();

                        let published_time = vr
                            .get("publishedTimeText")
                            .and_then(|p| p.get("simpleText"))
                            .and_then(|t| t.as_str())
                            .unwrap_or_default()
                            .to_string();

                        let description = vr
                            .get("descriptionSnippet")
                            .and_then(|d| d.get("runs"))
                            .and_then(|r| r.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                                    .collect::<Vec<_>>()
                                    .join("")
                            })
                            .unwrap_or_default();

                        let url = format!("https://www.youtube.com/watch?v={}", video_id);

                        if !title.is_empty() && !list.iter().any(|v| v.video_id == video_id) {
                            list.push(VideoResult {
                                title,
                                video_id,
                                url,
                                channel,
                                duration,
                                views,
                                published_time,
                                description,
                            });
                        }
                    }
                }
                for v in map.values() {
                    Self::extract_youtube_videos(v, list);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr {
                    Self::extract_youtube_videos(v, list);
                }
            }
            _ => {}
        }
    }
}
