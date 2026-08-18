use crate::dom::interactive::InteractiveElement;
use crate::render::css::{ComputedStyle, CssParser, Display, FlexDirection};
use scraper::{ElementRef, Html, Node};

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub enum LayoutContent {
    Element {
        tag: String,
        children: Vec<LayoutBox>,
    },
    Text(String),
    Image {
        src: String,
        alt: String,
    },
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub rect: Rect,
    pub style: ComputedStyle,
    pub content: LayoutContent,
    pub interactive_index: Option<usize>,
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn build_and_layout(
        html_str: &str,
        interactive: &[InteractiveElement],
        viewport_width: f32,
    ) -> LayoutBox {
        let document = Html::parse_document(html_str);
        let root_element = document.root_element();

        // 1. Build initial layout box tree
        let mut root_box = Self::build_tree(&root_element, interactive);

        // 2. Perform layout calculation pass
        let mut y_cursor = 70.0; // below top browser header bar
        let x_margin = 40.0;
        let content_width = (viewport_width - (x_margin * 2.0)).max(300.0);

        root_box.rect = Rect {
            x: 0.0,
            y: 0.0,
            width: viewport_width,
            height: 1200.0,
        };

        Self::layout_box(&mut root_box, x_margin, &mut y_cursor, content_width);

        root_box
    }

    fn build_tree(element: &ElementRef, interactive: &[InteractiveElement]) -> LayoutBox {
        let tag = element.value().name().to_string();
        let mut style = CssParser::default_style_for_tag(&tag);

        if let Some(inline_style) = element.value().attr("style") {
            CssParser::parse_inline_style(inline_style, &mut style);
        }

        // Check if matching interactive element
        let text_content = element
            .text()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();
        let interactive_index = interactive
            .iter()
            .find(|i| {
                (!i.text.is_empty() && i.text == text_content)
                    || (element
                        .value()
                        .attr("id")
                        .is_some_and(|id| id == i.name || i.selector.contains(id)))
                    || (element.value().attr("name").is_some_and(|n| n == i.name))
            })
            .map(|i| i.index);

        let mut children = Vec::new();

        if tag == "img" {
            let src = element.value().attr("src").unwrap_or("").to_string();
            let alt = element.value().attr("alt").unwrap_or("").to_string();
            return LayoutBox {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 320.0,
                    height: 180.0,
                },
                style,
                content: LayoutContent::Image { src, alt },
                interactive_index,
            };
        }

        for child in element.children() {
            match child.value() {
                Node::Element(_) => {
                    if let Some(child_el) = ElementRef::wrap(child) {
                        let child_box = Self::build_tree(&child_el, interactive);
                        if !child_box.style.is_hidden {
                            children.push(child_box);
                        }
                    }
                }
                Node::Text(txt) => {
                    let text = txt.text.trim().to_string();
                    if !text.is_empty() {
                        children.push(LayoutBox {
                            rect: Rect {
                                x: 0.0,
                                y: 0.0,
                                width: 0.0,
                                height: 0.0,
                            },
                            style: style.clone(),
                            content: LayoutContent::Text(text),
                            interactive_index: None,
                        });
                    }
                }
                _ => {}
            }
        }

        LayoutBox {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            style,
            content: LayoutContent::Element { tag, children },
            interactive_index,
        }
    }

    fn layout_box(box_node: &mut LayoutBox, x: f32, y_cursor: &mut f32, available_width: f32) {
        if box_node.style.is_hidden {
            return;
        }

        let start_x = x + box_node.style.margin_left + box_node.style.padding_left;
        let start_y = *y_cursor + box_node.style.margin_top;

        match &mut box_node.content {
            LayoutContent::Text(txt) => {
                let char_count = txt.chars().count();
                let approx_line_width = available_width.max(100.0);
                let chars_per_line =
                    (approx_line_width / (box_node.style.font_size * 0.55)).max(10.0) as usize;
                let lines_count = (char_count / chars_per_line).max(1);
                let height = lines_count as f32 * (box_node.style.font_size * 1.4);

                box_node.rect = Rect {
                    x: start_x,
                    y: start_y,
                    width: available_width,
                    height,
                };
                *y_cursor = start_y + height + box_node.style.margin_bottom;
            }
            LayoutContent::Image { .. } => {
                let width = box_node.style.width.unwrap_or(320.0).min(available_width);
                let height = box_node.style.height.unwrap_or(180.0);

                box_node.rect = Rect {
                    x: start_x,
                    y: start_y,
                    width,
                    height,
                };
                *y_cursor = start_y + height + box_node.style.margin_bottom;
            }
            LayoutContent::Element { children, .. } => {
                let element_width = box_node.style.width.unwrap_or(available_width);
                let mut current_child_y = start_y + box_node.style.padding_top;

                if box_node.style.display == Display::Flex
                    && box_node.style.flex_direction == FlexDirection::Row
                {
                    // Flex row layout
                    let mut child_x = start_x;
                    let mut max_row_height: f32 = 0.0;
                    let child_width = if !children.is_empty() {
                        (element_width - (box_node.style.gap * (children.len() as f32 - 1.0)))
                            / children.len() as f32
                    } else {
                        element_width
                    };

                    for child in children.iter_mut() {
                        let mut temp_y = current_child_y;
                        Self::layout_box(child, child_x, &mut temp_y, child_width);
                        let child_h = child.rect.height;
                        if child_h > max_row_height {
                            max_row_height = child_h;
                        }
                        child_x += child_width + box_node.style.gap;
                    }
                    current_child_y += max_row_height;
                } else {
                    // Block / normal vertical flow layout
                    for child in children.iter_mut() {
                        Self::layout_box(child, start_x, &mut current_child_y, element_width);
                    }
                }

                let total_height = (current_child_y - start_y) + box_node.style.padding_bottom;
                box_node.rect = Rect {
                    x: start_x,
                    y: start_y,
                    width: element_width,
                    height: total_height,
                };

                *y_cursor = start_y + total_height + box_node.style.margin_bottom;
            }
        }
    }
}
