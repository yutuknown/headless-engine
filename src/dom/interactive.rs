use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    pub index: usize,
    pub tag: String,
    pub role: String,
    pub text: String,
    pub name: String,
    pub input_type: String,
    pub placeholder: String,
    pub value: String,
    pub href: String,
    pub selector: String,
    pub is_clickable: bool,
    pub is_input: bool,
}

impl InteractiveElement {
    pub fn to_agent_string(&self) -> String {
        if self.is_input {
            let label = if !self.placeholder.is_empty() {
                format!("placeholder=\"{}\"", self.placeholder)
            } else if !self.name.is_empty() {
                format!("name=\"{}\"", self.name)
            } else {
                format!("type=\"{}\"", self.input_type)
            };
            format!(
                "[{}] <input {}> (value=\"{}\")",
                self.index, label, self.value
            )
        } else if self.tag == "button" || self.role == "button" {
            format!("[{}] <button \"{}\">", self.index, self.text)
        } else if self.tag == "a" {
            format!("[{}] <a \"{}\"> -> {}", self.index, self.text, self.href)
        } else {
            format!("[{}] <{} \"{}\">", self.index, self.tag, self.text)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageObservation {
    pub url: String,
    pub title: String,
    pub is_captcha_detected: bool,
    pub interactive_elements: Vec<InteractiveElement>,
    pub agent_tree_text: String,
    pub content_summary_markdown: String,
}

pub struct InteractiveParser;

impl InteractiveParser {
    pub fn parse(html_str: &str, base_url: Option<&str>) -> Vec<InteractiveElement> {
        let document = Html::parse_document(html_str);
        let mut elements = Vec::new();

        let query = "a[href], button, input, textarea, select, [contenteditable='true'], [role='button'], [role='link'], [role='checkbox'], [role='tab'], [onclick], [data-testid]";
        let selector = match Selector::parse(query) {
            Ok(s) => s,
            Err(_) => return elements,
        };

        let mut index = 1;
        for el in document.select(&selector) {
            let tag = el.value().name().to_string();
            let role = el.value().attr("role").unwrap_or("").to_string();
            let name = el.value().attr("name").unwrap_or("").to_string();
            let input_type = el.value().attr("type").unwrap_or("text").to_string();
            let placeholder = el.value().attr("placeholder").unwrap_or("").to_string();
            let value = el.value().attr("value").unwrap_or("").to_string();
            let id = el.value().attr("id").unwrap_or("").to_string();
            let class = el.value().attr("class").unwrap_or("").to_string();
            let is_contenteditable = el.value().attr("contenteditable") == Some("true");

            // Skip hidden inputs and disabled elements
            if input_type == "hidden"
                || el.value().attr("disabled").is_some()
                || el.value().attr("aria-hidden") == Some("true")
            {
                continue;
            }

            let mut text = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
            if text.is_empty() {
                if let Some(aria_label) = el
                    .value()
                    .attr("aria-label")
                    .or_else(|| el.value().attr("title"))
                {
                    text = aria_label.trim().to_string();
                }
            }

            // Resolve href for links
            let raw_href = el.value().attr("href").unwrap_or("");
            let href = if !raw_href.is_empty() && !raw_href.starts_with("javascript:") {
                Self::resolve_url(raw_href, base_url)
            } else {
                String::new()
            };

            // Skip empty non-input elements
            if text.is_empty()
                && href.is_empty()
                && placeholder.is_empty()
                && name.is_empty()
                && id.is_empty()
                && !is_contenteditable
            {
                continue;
            }

            let is_input = tag == "input"
                || tag == "textarea"
                || tag == "select"
                || is_contenteditable
                || id == "prompt-textarea";
            let is_clickable = tag == "a"
                || tag == "button"
                || role == "button"
                || role == "link"
                || el.value().attr("onclick").is_some();

            // Build unique CSS selector
            let css_selector = if !id.is_empty() {
                format!("#{}", id)
            } else if !name.is_empty() {
                format!("{}[name='{}']", tag, name)
            } else if !class.is_empty() {
                let first_class = class.split_whitespace().next().unwrap_or("");
                format!("{}.{}", tag, first_class)
            } else {
                tag.clone()
            };

            elements.push(InteractiveElement {
                index,
                tag,
                role,
                text,
                name,
                input_type,
                placeholder,
                value,
                href,
                selector: css_selector,
                is_clickable,
                is_input,
            });

            index += 1;
        }

        elements
    }

    fn resolve_url(href: &str, base_url: Option<&str>) -> String {
        if href.starts_with("http://") || href.starts_with("https://") {
            return href.to_string();
        }
        if let Some(base) = base_url {
            if href.starts_with("//") {
                return format!("https:{}", href);
            }
            if href.starts_with('/') {
                if let Some(idx) = base.find("://") {
                    let after = &base[idx + 3..];
                    let host = after.split('/').next().unwrap_or(after);
                    let scheme = &base[..idx + 3];
                    return format!("{}{}{}", scheme, host, href);
                }
            }
            let trimmed_base = base.split('?').next().unwrap_or(base);
            let parent = if trimmed_base.ends_with('/') {
                trimmed_base
            } else if let Some(last_slash) = trimmed_base.rfind('/') {
                &trimmed_base[..last_slash + 1]
            } else {
                trimmed_base
            };
            return format!("{}{}", parent, href);
        }
        href.to_string()
    }
}
