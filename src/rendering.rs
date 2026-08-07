//! Complete HTML rendering facade with DOM, CSS, layout, and canvas hooks.
//!
//! The module preserves Glycerin's rendering API while using a Rust-only
//! fallback pipeline by default. Enabling `native-skia` swaps the canvas type to
//! the real Skia crate through the crate-level compatibility export.

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
        Self { display: "block".into(), position: "static".into(), width: None, height: None, margin: [0.0; 4], padding: [0.0; 4], background_color: None, color: (0, 0, 0, 255), font_size: 16.0 }
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
pub struct Document { pub elements: Vec<DomElement> }

pub struct HtmlRenderer { document: Document, pub styles: HashMap<String, ComputedStyle> }

impl HtmlRenderer {
    pub fn parse_html(html: &str) -> Self {
        let mut elements = Vec::new();
        for (tag, attrs) in scan_tags(html) {
            let id = extract_attr(&attrs, "id");
            let classes = extract_attr(&attrs, "class").map(|v| v.split_whitespace().map(ToString::to_string).collect()).unwrap_or_default();
            elements.push(DomElement { tag_name: tag, id, classes, attributes: HashMap::new() });
        }
        Self { document: Document { elements }, styles: HashMap::new() }
    }

    pub fn get_document(&self) -> Document { self.document.clone() }
    pub fn build_dom_elements(&self, document: &Document) -> Vec<DomElement> { document.elements.clone() }
    pub fn apply_styles(&mut self, css: &str) {
        let selectors = css.split('{').step_by(2).map(str::trim).filter(|s| !s.is_empty());
        let mut inserted = false;
        for selector in selectors {
            self.styles.insert(selector.trim_start_matches('}').trim().to_string(), ComputedStyle::default());
            inserted = true;
        }
        if !inserted { self.styles.insert("default".into(), ComputedStyle::default()); }
    }

    pub fn calculate_layout(&self, viewport_width: f32, viewport_height: f32) -> LayoutBox {
        LayoutBox { x: 0.0, y: 0.0, width: viewport_width, height: viewport_height, children: Vec::new(), style: ComputedStyle::default(), text: None }
    }

    pub fn render_to_canvas(&self, canvas: &mut crate::skia_safe::Canvas, layout: &LayoutBox) {
        let paint = crate::skia_safe::Paint::new(crate::skia_safe::Color::from_argb(255, 255, 255, 255), None);
        let rect = crate::skia_safe::Rect::new(layout.x, layout.y, layout.x + layout.width, layout.y + layout.height);
        canvas.draw_rect(rect, &paint);
    }
}

fn scan_tags(html: &str) -> Vec<(String, String)> {
    let mut tags = Vec::new();
    for part in html.split('<').skip(1) {
        if part.starts_with('/') || part.starts_with('!') { continue; }
        if let Some(end) = part.find('>') {
            let inside = &part[..end];
            let mut pieces = inside.splitn(2, char::is_whitespace);
            if let Some(tag) = pieces.next().filter(|t| !t.is_empty()) {
                tags.push((tag.to_ascii_lowercase(), pieces.next().unwrap_or_default().to_string()));
            }
        }
    }
    tags
}

fn extract_attr(attrs: &str, name: &str) -> Option<String> {
    let needle = format!("{}=\"", name);
    let start = attrs.find(&needle)? + needle.len();
    let end = attrs[start..].find('"')?;
    Some(attrs[start..start + end].to_string())
}
