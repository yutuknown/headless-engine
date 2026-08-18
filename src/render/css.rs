#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub display: Display,
    pub color: String,
    pub background_color: Option<String>,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub border_radius: f32,
    pub border_color: Option<String>,
    pub border_width: f32,
    pub flex_direction: FlexDirection,
    pub gap: f32,
    pub is_hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    InlineBlock,
    Flex,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Normal,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::Block,
            color: "#e2e8f0".to_string(), // Slate-200 default
            background_color: None,
            font_size: 14.0,
            font_weight: FontWeight::Normal,
            width: None,
            height: None,
            margin_top: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            margin_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            padding_right: 0.0,
            border_radius: 0.0,
            border_color: None,
            border_width: 0.0,
            flex_direction: FlexDirection::Row,
            gap: 0.0,
            is_hidden: false,
        }
    }
}

pub struct CssParser;

impl CssParser {
    pub fn default_style_for_tag(tag: &str) -> ComputedStyle {
        let mut s = ComputedStyle::default();
        match tag {
            "html" | "body" => {
                s.display = Display::Block;
                s.background_color = Some("#0f172a".to_string()); // Default dark slate theme
                s.color = "#f8fafc".to_string();
                s.font_size = 14.0;
            }
            "h1" => {
                s.display = Display::Block;
                s.font_size = 26.0;
                s.font_weight = FontWeight::Bold;
                s.color = "#f8fafc".to_string();
                s.margin_top = 16.0;
                s.margin_bottom = 12.0;
            }
            "h2" => {
                s.display = Display::Block;
                s.font_size = 20.0;
                s.font_weight = FontWeight::Bold;
                s.color = "#38bdf8".to_string();
                s.margin_top = 14.0;
                s.margin_bottom = 8.0;
            }
            "h3" => {
                s.display = Display::Block;
                s.font_size = 16.0;
                s.font_weight = FontWeight::Bold;
                s.color = "#e2e8f0".to_string();
                s.margin_top = 10.0;
                s.margin_bottom = 6.0;
            }
            "p" => {
                s.display = Display::Block;
                s.font_size = 14.0;
                s.color = "#cbd5e1".to_string();
                s.margin_bottom = 10.0;
            }
            "a" => {
                s.display = Display::Inline;
                s.color = "#60a5fa".to_string(); // Blue link
                s.font_size = 14.0;
            }
            "button" => {
                s.display = Display::InlineBlock;
                s.background_color = Some("#3b82f6".to_string());
                s.color = "#ffffff".to_string();
                s.font_weight = FontWeight::Bold;
                s.padding_top = 6.0;
                s.padding_bottom = 6.0;
                s.padding_left = 14.0;
                s.padding_right = 14.0;
                s.border_radius = 6.0;
                s.margin_right = 8.0;
                s.margin_bottom = 8.0;
            }
            "input" | "textarea" => {
                s.display = Display::InlineBlock;
                s.background_color = Some("#1e293b".to_string());
                s.color = "#f8fafc".to_string();
                s.border_color = Some("#475569".to_string());
                s.border_width = 1.0;
                s.border_radius = 6.0;
                s.padding_top = 6.0;
                s.padding_bottom = 6.0;
                s.padding_left = 10.0;
                s.padding_right = 10.0;
                s.width = Some(240.0);
                s.margin_right = 8.0;
                s.margin_bottom = 8.0;
            }
            "img" => {
                s.display = Display::InlineBlock;
                s.border_radius = 6.0;
            }
            "header" | "nav" => {
                s.display = Display::Block;
                s.background_color = Some("#1e293b".to_string());
                s.padding_top = 12.0;
                s.padding_bottom = 12.0;
                s.padding_left = 16.0;
                s.padding_right = 16.0;
                s.margin_bottom = 16.0;
            }
            "table" => {
                s.display = Display::Block;
                s.margin_top = 10.0;
                s.margin_bottom = 14.0;
                s.border_width = 1.0;
                s.border_color = Some("#334155".to_string());
            }
            "tr" => {
                s.display = Display::Flex;
                s.flex_direction = FlexDirection::Row;
            }
            "th" | "td" => {
                s.display = Display::Block;
                s.padding_top = 6.0;
                s.padding_bottom = 6.0;
                s.padding_left = 10.0;
                s.padding_right = 10.0;
                s.border_width = 1.0;
                s.border_color = Some("#334155".to_string());
            }
            "script" | "style" | "noscript" | "meta" | "head" | "link" => {
                s.display = Display::None;
                s.is_hidden = true;
            }
            _ => {
                // span, div, section, article, etc.
                s.display = if tag == "span" || tag == "b" || tag == "i" || tag == "strong" {
                    Display::Inline
                } else {
                    Display::Block
                };
            }
        }
        s
    }

    pub fn parse_inline_style(style_str: &str, base_style: &mut ComputedStyle) {
        for rule in style_str.split(';') {
            let rule = rule.trim();
            if rule.is_empty() {
                continue;
            }
            let mut parts = rule.splitn(2, ':');
            let prop = parts.next().unwrap_or("").trim().to_lowercase();
            let val = parts.next().unwrap_or("").trim();

            match prop.as_str() {
                "display" => match val {
                    "none" => {
                        base_style.display = Display::None;
                        base_style.is_hidden = true;
                    }
                    "flex" => base_style.display = Display::Flex,
                    "inline" => base_style.display = Display::Inline,
                    "inline-block" => base_style.display = Display::InlineBlock,
                    "block" => base_style.display = Display::Block,
                    _ => {}
                },
                "color" => {
                    base_style.color = Self::parse_color(val);
                }
                "background-color" | "background" => {
                    if !val.contains("url") {
                        base_style.background_color = Some(Self::parse_color(val));
                    }
                }
                "font-size" => {
                    if let Some(px) = Self::parse_pixels(val) {
                        base_style.font_size = px;
                    }
                }
                "font-weight" => {
                    if val == "bold" || val == "700" || val == "800" || val == "900" {
                        base_style.font_weight = FontWeight::Bold;
                    } else {
                        base_style.font_weight = FontWeight::Normal;
                    }
                }
                "width" => {
                    base_style.width = Self::parse_pixels(val);
                }
                "height" => {
                    base_style.height = Self::parse_pixels(val);
                }
                "margin" => {
                    if let Some(px) = Self::parse_pixels(val) {
                        base_style.margin_top = px;
                        base_style.margin_bottom = px;
                        base_style.margin_left = px;
                        base_style.margin_right = px;
                    }
                }
                "padding" => {
                    if let Some(px) = Self::parse_pixels(val) {
                        base_style.padding_top = px;
                        base_style.padding_bottom = px;
                        base_style.padding_left = px;
                        base_style.padding_right = px;
                    }
                }
                "border-radius" => {
                    if let Some(px) = Self::parse_pixels(val) {
                        base_style.border_radius = px;
                    }
                }
                "border" => {
                    base_style.border_width = 1.0;
                    base_style.border_color = Some("#475569".to_string());
                }
                "flex-direction" => {
                    if val == "column" {
                        base_style.flex_direction = FlexDirection::Column;
                    } else {
                        base_style.flex_direction = FlexDirection::Row;
                    }
                }
                "gap" => {
                    if let Some(px) = Self::parse_pixels(val) {
                        base_style.gap = px;
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_pixels(val: &str) -> Option<f32> {
        let clean = val
            .trim()
            .trim_end_matches("px")
            .trim_end_matches("em")
            .trim_end_matches("rem");
        clean.parse::<f32>().ok()
    }

    fn parse_color(val: &str) -> String {
        let val = val.trim();
        if val.starts_with('#') || val.starts_with("rgb") || val.starts_with("hsl") {
            val.to_string()
        } else {
            match val {
                "white" => "#ffffff".to_string(),
                "black" => "#000000".to_string(),
                "red" => "#ef4444".to_string(),
                "blue" => "#3b82f6".to_string(),
                "green" => "#10b981".to_string(),
                "gray" | "grey" => "#64748b".to_string(),
                _ => val.to_string(),
            }
        }
    }
}
