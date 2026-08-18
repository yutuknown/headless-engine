use crate::render::layout::{LayoutBox, LayoutContent};
use anyhow::Result;
use base64::Engine;
use std::collections::HashMap;

pub struct PaintEngine;

impl PaintEngine {
    pub async fn paint_to_svg_and_png(
        url: &str,
        title: &str,
        root_box: &LayoutBox,
        width: u32,
        height: u32,
    ) -> Result<(String, Vec<u8>)> {
        let mut svg_body = String::new();

        // 1. Browser Chrome Header Bar
        let url_escaped = Self::xml_escape(url);
        let title_escaped = Self::xml_escape(&title.chars().take(35).collect::<String>());

        let header = format!(
            "  <!-- Browser Navigation Bar -->\n\
  <rect width=\"{width}\" height=\"56\" fill=\"#1e293b\" />\n\
  <circle cx=\"22\" cy=\"28\" r=\"6\" fill=\"#ef4444\" />\n\
  <circle cx=\"40\" cy=\"28\" r=\"6\" fill=\"#f59e0b\" />\n\
  <circle cx=\"58\" cy=\"28\" r=\"6\" fill=\"#10b981\" />\n\
  <rect x=\"80\" y=\"12\" width=\"800\" height=\"32\" rx=\"16\" fill=\"#0f172a\" stroke=\"#334155\" stroke-width=\"1\" />\n\
  <text x=\"98\" y=\"32\" fill=\"#94a3b8\" font-size=\"12\">&#128274; {url_escaped}</text>\n\
  <text x=\"900\" y=\"32\" fill=\"#e2e8f0\" font-size=\"12\" font-weight=\"600\">{title_escaped}</text>\n\
  <line x1=\"0\" y1=\"56\" x2=\"{width}\" y2=\"56\" stroke=\"#334155\" stroke-width=\"1\" />\n\
  \n\
  <!-- Webpage Render Canvas -->\n\
  <g transform=\"translate(0, 56)\">\n"
        );
        svg_body.push_str(&header);

        // 2. Fetch images concurrently if any
        let mut image_urls = Vec::new();
        Self::collect_image_urls(root_box, &mut image_urls);

        let mut image_cache: HashMap<String, String> = HashMap::new();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(2000))
            .build()
            .ok();

        if let Some(ref c) = client {
            for img_url in image_urls.into_iter().take(12) {
                if img_url.starts_with("http://") || img_url.starts_with("https://") {
                    if let Ok(resp) = c.get(&img_url).send().await {
                        if let Ok(bytes) = resp.bytes().await {
                            let mime = if img_url.ends_with(".png") {
                                "image/png"
                            } else if img_url.ends_with(".webp") {
                                "image/webp"
                            } else {
                                "image/jpeg"
                            };
                            let b64 = format!(
                                "data:{};base64,{}",
                                mime,
                                base64::engine::general_purpose::STANDARD.encode(&bytes)
                            );
                            image_cache.insert(img_url, b64);
                        }
                    }
                }
            }
        }

        // 3. Paint layout box tree recursively
        Self::paint_box(root_box, &mut svg_body, &image_cache);

        svg_body.push_str("  </g>\n</svg>");

        let full_svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\" style=\"background:#0f172a; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;\">\n\
{body}",
            width = width,
            height = height,
            body = svg_body
        );

        // 4. Rasterize to real PNG via pure-Rust resvg + tiny-skia
        let opt = resvg::usvg::Options {
            font_family: "sans-serif".to_string(),
            ..Default::default()
        };
        let tree = resvg::usvg::Tree::from_str(&full_svg, &opt)?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| anyhow::anyhow!("Failed to allocate raster buffer"))?;

        resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
        let png_bytes = pixmap.encode_png()?;

        Ok((full_svg, png_bytes))
    }

    fn collect_image_urls(box_node: &LayoutBox, urls: &mut Vec<String>) {
        match &box_node.content {
            LayoutContent::Image { src, .. } => {
                if !src.is_empty() {
                    urls.push(src.clone());
                }
            }
            LayoutContent::Element { children, .. } => {
                for child in children {
                    Self::collect_image_urls(child, urls);
                }
            }
            _ => {}
        }
    }

    fn paint_box(box_node: &LayoutBox, svg: &mut String, image_cache: &HashMap<String, String>) {
        if box_node.style.is_hidden || box_node.rect.y > 1200.0 {
            return;
        }

        let r = &box_node.rect;

        // Draw Background
        if let Some(ref bg) = box_node.style.background_color {
            if r.width > 0.0 && r.height > 0.0 {
                svg.push_str(&format!(
                    "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\" />\n",
                    r.x, r.y, r.width, r.height, box_node.style.border_radius, bg
                ));
            }
        }

        // Draw Border
        if let Some(ref bc) = box_node.style.border_color {
            if box_node.style.border_width > 0.0 {
                svg.push_str(&format!(
                    "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" />\n",
                    r.x, r.y, r.width, r.height, box_node.style.border_radius, bc, box_node.style.border_width
                ));
            }
        }

        // Draw Content
        match &box_node.content {
            LayoutContent::Text(txt) => {
                if !txt.is_empty() {
                    let escaped = Self::xml_escape(txt);
                    let weight = if box_node.style.font_weight == crate::render::css::FontWeight::Bold {
                        " font-weight=\"bold\""
                    } else {
                        ""
                    };

                    let badge = if let Some(idx) = box_node.interactive_index {
                        format!("[{}] ", idx)
                    } else {
                        String::new()
                    };

                    svg.push_str(&format!(
                        "    <text x=\"{}\" y=\"{}\" fill=\"{}\" font-size=\"{}\"{}>{}{}</text>\n",
                        r.x,
                        r.y + (box_node.style.font_size * 0.9),
                        box_node.style.color,
                        box_node.style.font_size,
                        weight,
                        badge,
                        escaped
                    ));
                }
            }
            LayoutContent::Image { src, alt: _ } => {
                if let Some(data_url) = image_cache.get(src) {
                    svg.push_str(&format!(
                        "    <image href=\"{}\" xlink:href=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"xMidYMid slice\" />\n",
                        data_url, data_url, r.x, r.y, r.width, r.height
                    ));
                } else {
                    svg.push_str(&format!(
                        "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"6\" fill=\"#1e293b\" stroke=\"#334155\" stroke-width=\"1\" />\n\
                             <text x=\"{}\" y=\"{}\" fill=\"#64748b\" font-size=\"11\">[Image]</text>\n",
                        r.x, r.y, r.width, r.height,
                        r.x + 10.0, r.y + 20.0
                    ));
                }
            }
            LayoutContent::Element { children, .. } => {
                // Paint children in document order
                for child in children {
                    Self::paint_box(child, svg, image_cache);
                }
            }
        }
    }

    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
