use scraper::node::Node;
use scraper::{ElementRef, Html, Selector};

pub struct HtmlToMarkdown;

impl HtmlToMarkdown {
    pub fn convert(html_str: &str, base_url: Option<&str>) -> String {
        let document = Html::parse_document(html_str);

        // Try extracting main content container if available to eliminate boilerplate
        let content_selector = Selector::parse(
            "article, main, div[role='main'], div.main-content, div.post-content, div#content, body",
        )
        .ok();

        let root_element = content_selector
            .as_ref()
            .and_then(|sel| document.select(sel).next())
            .unwrap_or_else(|| document.root_element());

        let mut output = String::new();
        Self::walk_node(&root_element, &mut output, base_url);

        Self::clean_markdown(&output)
    }

    pub fn convert_element(element: &ElementRef, base_url: Option<&str>) -> String {
        let mut output = String::new();
        Self::walk_node(element, &mut output, base_url);
        Self::clean_markdown(&output)
    }

    fn walk_node(element: &ElementRef, output: &mut String, base_url: Option<&str>) {
        let tag_name = element.value().name();

        // Skip non-content / hidden tags
        match tag_name {
            "script" | "style" | "noscript" | "svg" | "canvas" | "iframe" | "header" | "footer"
            | "nav" | "dialog" => return,
            _ => {}
        }

        match tag_name {
            "h1" => output.push_str("\n\n# "),
            "h2" => output.push_str("\n\n## "),
            "h3" => output.push_str("\n\n### "),
            "h4" => output.push_str("\n\n#### "),
            "h5" => output.push_str("\n\n##### "),
            "h6" => output.push_str("\n\n###### "),
            "p" => output.push_str("\n\n"),
            "br" => output.push('\n'),
            "hr" => output.push_str("\n\n---\n\n"),
            "blockquote" => output.push_str("\n\n> "),
            "li" => output.push_str("\n- "),
            "pre" => output.push_str("\n\n```\n"),
            "code" => output.push('`'),
            "strong" | "b" => output.push_str("**"),
            "em" | "i" => output.push('*'),
            "a" => {
                let href = element.value().attr("href").unwrap_or("");
                let full_url = Self::resolve_url(href, base_url);
                let text = element
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
                if !text.is_empty() && !full_url.is_empty() && !full_url.starts_with("javascript:")
                {
                    output.push_str(&format!("[{}]({})", text, full_url));
                    return; // Handled children
                }
            }
            "img" => {
                let src = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                    .unwrap_or("");
                let alt = element.value().attr("alt").unwrap_or("image");
                let full_url = Self::resolve_url(src, base_url);
                if !full_url.is_empty() {
                    output.push_str(&format!("![{}]({})", alt, full_url));
                }
                return;
            }
            "table" => {
                Self::render_table(element, output);
                return;
            }
            _ => {}
        }

        for child in element.children() {
            match child.value() {
                Node::Element(_) => {
                    if let Some(child_ref) = ElementRef::wrap(child) {
                        Self::walk_node(&child_ref, output, base_url);
                    }
                }
                Node::Text(text) => {
                    let raw = text.as_ref();
                    if tag_name == "pre" || tag_name == "code" {
                        output.push_str(raw);
                    } else {
                        let trimmed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
                        if !trimmed.is_empty() {
                            if raw.starts_with(char::is_whitespace)
                                && !output.ends_with(' ')
                                && !output.ends_with('\n')
                            {
                                output.push(' ');
                            }
                            output.push_str(&trimmed);
                            if raw.ends_with(char::is_whitespace) {
                                output.push(' ');
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        match tag_name {
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "blockquote" => output.push_str("\n\n"),
            "pre" => output.push_str("\n```\n\n"),
            "code" => output.push('`'),
            "strong" | "b" => output.push_str("**"),
            "em" | "i" => output.push('*'),
            _ => {}
        }
    }

    fn render_table(table: &ElementRef, output: &mut String) {
        let row_sel = Selector::parse("tr").expect("valid static selector");
        let cell_sel = Selector::parse("th, td").expect("valid static selector");

        let mut rows = Vec::new();
        for tr in table.select(&row_sel) {
            let cells: Vec<String> = tr
                .select(&cell_sel)
                .map(|c| c.text().collect::<Vec<_>>().join(" ").trim().to_string())
                .collect();
            if !cells.is_empty() {
                rows.push(cells);
            }
        }

        if rows.is_empty() {
            return;
        }

        output.push_str("\n\n");
        let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);

        // Header row
        if let Some(headers) = rows.first() {
            output.push_str("| ");
            for i in 0..max_cols {
                let val = headers.get(i).map(|s| s.as_str()).unwrap_or("");
                output.push_str(val);
                output.push_str(" | ");
            }
            output.push('\n');

            // Separator
            output.push_str("| ");
            for _ in 0..max_cols {
                output.push_str("--- | ");
            }
            output.push('\n');
        }

        // Body rows
        for row in rows.iter().skip(1) {
            output.push_str("| ");
            for i in 0..max_cols {
                let val = row.get(i).map(|s| s.as_str()).unwrap_or("");
                output.push_str(val);
                output.push_str(" | ");
            }
            output.push('\n');
        }
        output.push('\n');
    }

    fn resolve_url(href: &str, base_url: Option<&str>) -> String {
        if href.is_empty() {
            return String::new();
        }
        if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("data:")
        {
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

    fn clean_markdown(input: &str) -> String {
        let mut result = String::new();
        let mut newline_count = 0;

        for line in input.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if newline_count < 2 {
                    result.push('\n');
                    newline_count += 1;
                }
            } else {
                if newline_count > 0 && !result.is_empty() && !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push_str(trimmed);
                result.push('\n');
                newline_count = 0;
            }
        }

        result.trim().to_string()
    }
}
