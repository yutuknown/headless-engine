pub mod css;
pub mod layout;
pub mod paint;

use crate::dom::interactive::InteractiveElement;
use crate::dom::screenshot::ScreenshotResult;
use anyhow::Result;
use base64::Engine;
pub use layout::LayoutEngine;
pub use paint::PaintEngine;

pub struct HtmlRenderer;

impl HtmlRenderer {
    pub async fn render_html_to_screenshot(
        url: &str,
        title: &str,
        html_str: &str,
        interactive: &[InteractiveElement],
        width: u32,
        height: u32,
    ) -> Result<ScreenshotResult> {
        let root_box = LayoutEngine::build_and_layout(html_str, interactive, width as f32);
        let (svg, png_bytes) = PaintEngine::paint_to_svg_and_png(url, title, &root_box, width, height).await?;

        let png_base64 = if !png_bytes.is_empty() {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&png_bytes)
            )
        } else {
            String::new()
        };

        Ok(ScreenshotResult {
            width,
            height,
            svg,
            layout_wireframe: String::new(),
            element_count: interactive.len(),
            png_bytes,
            png_base64,
        })
    }
}
