//! Lightweight HTML rendering facade used by the browser shell.

use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct DomElement {
    pub tag_name: String,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct ComputedStyle {
    pub display: String,
    pub position: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub margin: [f32; 4],
    pub padding: [f32; 4],
    pub background_color: Option<(u8, u8, u8, u8)>,
    pub color: (u8, u8, u8, u8),
    pub font_size: f32,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: "block".to_string(),
            position: "static".to_string(),
            width: None,
            height: None,
            margin: [0.0; 4],
            padding: [0.0; 4],
            background_color: None,
            color: (0, 0, 0, 255),
            font_size: 16.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LayoutBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub children: Vec<LayoutBox>,
    pub style: ComputedStyle,
    pub text: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Document {
    pub elements: Vec<DomElement>,
}

pub struct HtmlRenderer {
    document: Document,
    pub styles: HashMap<String, ComputedStyle>,
}

impl HtmlRenderer {
    pub fn parse_html(html: &str) -> Self {
        let mut elements = Vec::new();
        for tag in ["html", "head", "title", "body", "header", "main", "article", "footer", "div", "h1", "h2", "p"] {
            for attrs in html.match_indices(&format!("<{}", tag)) {
                let rest = &html[attrs.0..html[attrs.0..].find('>').map(|i| attrs.0 + i).unwrap_or(html.len())];
                let id = extract_attr(rest, "id");
                let classes = extract_attr(rest, "class")
                    .map(|v| v.split_whitespace().map(ToString::to_string).collect())
                    .unwrap_or_default();
                elements.push(DomElement { tag_name: tag.to_string(), id, classes, attributes: HashMap::new() });
            }
        }
        Self { document: Document { elements }, styles: HashMap::new() }
    }

    pub fn get_document(&self) -> Document { self.document.clone() }

    pub fn build_dom_elements(&self, document: &Document) -> Vec<DomElement> { document.elements.clone() }

    pub fn apply_styles(&mut self, css: &str) {
        if css.trim().is_empty() {
            self.styles.insert("default".to_string(), ComputedStyle::default());
        } else {
            for selector in css.split('{').step_by(2) {
                let selector = selector.trim().trim_start_matches('}').trim();
                if !selector.is_empty() {
                    self.styles.insert(selector.to_string(), ComputedStyle::default());
                }
            }
        }
    }

    pub fn calculate_layout(&self, viewport_width: f32, viewport_height: f32) -> LayoutBox {
        LayoutBox { x: 0.0, y: 0.0, width: viewport_width, height: viewport_height, children: Vec::new(), style: ComputedStyle::default(), text: None }
    }

    pub fn render_to_canvas(&self, _canvas: &mut crate::skia_safe::Canvas, _layout: &LayoutBox) {}
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')?;
    Some(tag[start..start + end].to_string())
}
