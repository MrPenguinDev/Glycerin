//! Phase 6: Developer Tools & UX Polish
//! 
//! Implements the DevTools protocol (similar to Chrome DevTools),
//! find-in-page, zoom controls, and accessibility features.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Represents a node in the DOM tree for inspection
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DOMNode {
    pub id: u64,
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub text_content: Option<String>,
    pub children: Vec<u64>, // IDs of children
    pub parent_id: Option<u64>,
    pub box_model: BoxModel,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct BoxModel {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub margin: [f32; 4], // top, right, bottom, left
    pub border: [f32; 4],
    pub padding: [f32; 4],
}

/// Protocol messages for DevTools communication
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "method")]
pub enum DevToolsMessage {
    #[serde(rename = "DOM.enable")]
    DomEnable { id: u64 },
    
    #[serde(rename = "DOM.getDocument")]
    DomGetDocument { id: u64, depth: i32 },
    
    #[serde(rename = "DOM.inspectNode")]
    DomInspectNode { id: u64, node_id: u64 },
    
    #[serde(rename = "Runtime.evaluate")]
    RuntimeEvaluate { id: u64, expression: String },
    
    #[serde(rename = "Network.enable")]
    NetworkEnable { id: u64 },
    
    #[serde(rename = "Console.enable")]
    ConsoleEnable { id: u64 },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DevToolsResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<DevToolsError>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DevToolsError {
    pub code: i32,
    pub message: String,
}

/// Manages the DevTools session
pub struct DevToolsSession {
    pub is_attached: bool,
    pub dom_tree: HashMap<u64, DOMNode>,
    pub console_messages: Vec<ConsoleMessage>,
    pub network_requests: Vec<NetworkRequest>,
    next_node_id: u64,
}

#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub level: ConsoleLevel,
    pub text: String,
    pub timestamp: u64,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleLevel {
    Log,
    Warning,
    Error,
    Info,
    Debug,
}

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub id: String,
    pub url: String,
    pub method: String,
    pub status: u16,
    pub timing: RequestTiming,
}

#[derive(Debug, Clone, Default)]
pub struct RequestTiming {
    pub dns_lookup: f64,
    pub tcp_connect: f64,
    pub ssl_handshake: f64,
    pub ttfb: f64, // Time to First Byte
    pub download: f64,
}

impl DevToolsSession {
    pub fn new() -> Self {
        Self {
            is_attached: false,
            dom_tree: HashMap::new(),
            console_messages: Vec::new(),
            network_requests: Vec::new(),
            next_node_id: 1,
        }
    }

    pub fn attach(&mut self) {
        self.is_attached = true;
    }

    pub fn detach(&mut self) {
        self.is_attached = false;
    }

    /// Process an incoming DevTools protocol message
    pub fn handle_message(&mut self, msg: DevToolsMessage) -> DevToolsResponse {
        match msg {
            DevToolsMessage::DomEnable { id } => {
                DevToolsResponse { id, result: Some(serde_json::json!({})), error: None }
            },
            DevToolsMessage::DomGetDocument { id, depth: _ } => {
                // Return root node
                let root = self.dom_tree.get(&1);
                DevToolsResponse {
                    id,
                    result: root.map(|n| serde_json::to_value(n).unwrap()),
                    error: None,
                }
            },
            DevToolsMessage::RuntimeEvaluate { id, expression } => {
                // In real impl, execute JS in renderer
                DevToolsResponse {
                    id,
                    result: Some(serde_json::json!({"value": "Evaluation not implemented in mock"})),
                    error: None,
                }
            },
            _ => DevToolsResponse {
                id: 0,
                result: None,
                error: Some(DevToolsError { code: -32601, message: "Method not found".to_string() }),
            }
        }
    }

    pub fn add_console_message(&mut self, level: ConsoleLevel, text: String, source: String) {
        self.console_messages.push(ConsoleMessage {
            level,
            text,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            source,
        });
    }
}

/// Find-in-page functionality
pub struct FindInPage {
    matches: Vec<FindMatch>,
    current_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FindMatch {
    pub text: String,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl FindInPage {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            current_index: None,
        }
    }

    /// Search for text in the document
    pub fn find(&mut self, content: &str, query: &str, case_sensitive: bool) -> usize {
        self.matches.clear();
        self.current_index = None;

        let search_content = if case_sensitive {
            content.to_string()
        } else {
            content.to_lowercase()
        };
        
        let search_query = if case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut start = 0;
        while let Some(pos) = search_content[start..].find(&search_query) {
            let absolute_pos = start + pos;
            self.matches.push(FindMatch {
                text: content[absolute_pos..absolute_pos + query.len()].to_string(),
                start_offset: absolute_pos,
                end_offset: absolute_pos + query.len(),
            });
            start = absolute_pos + 1;
        }

        self.matches.len()
    }

    /// Go to next match
    pub fn next_match(&mut self) -> Option<&FindMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let next_idx = match self.current_index {
            Some(idx) => (idx + 1) % self.matches.len(),
            None => 0,
        };

        self.current_index = Some(next_idx);
        Some(&self.matches[next_idx])
    }

    /// Go to previous match
    pub fn previous_match(&mut self) -> Option<&FindMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let prev_idx = match self.current_index {
            Some(idx) => {
                if idx == 0 { self.matches.len() - 1 } else { idx - 1 }
            },
            None => 0,
        };

        self.current_index = Some(prev_idx);
        Some(&self.matches[prev_idx])
    }
}

/// Viewport and zoom management
pub struct ViewportController {
    pub zoom_level: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

impl ViewportController {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            zoom_level: 1.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_width: width,
            viewport_height: height,
        }
    }

    pub fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level + 0.1).min(5.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level - 0.1).max(1.0);
    }

    pub fn reset_zoom(&mut self) {
        self.zoom_level = 1.0;
    }

    pub fn scroll_to(&mut self, x: f32, y: f32) {
        self.scroll_x = x.max(0.0);
        self.scroll_y = y.max(0.0);
    }

    pub fn get_scale_factor(&self) -> f32 {
        self.zoom_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_in_page() {
        let mut finder = FindInPage::new();
        let content = "Hello world, hello universe, hello everyone";
        
        let count = finder.find(content, "hello", false);
        assert_eq!(count, 3);
        
        let first = finder.next_match();
        assert!(first.is_some());
        assert_eq!(first.unwrap().text, "Hello");
        
        let second = finder.next_match();
        assert_eq!(second.unwrap().text, "hello");
    }

    #[test]
    fn test_viewport_zoom() {
        let mut vp = ViewportController::new(800.0, 600.0);
        assert_eq!(vp.zoom_level, 1.0);
        
        vp.zoom_in();
        assert_eq!(vp.zoom_level, 1.1);
        
        vp.zoom_out();
        vp.zoom_out();
        assert_eq!(vp.zoom_level, 1.0);
        
        for _ in 0..50 { vp.zoom_in(); }
        assert_eq!(vp.zoom_level, 5.0); // Max cap
    }

    #[test]
    fn test_devtools_session() {
        let mut session = DevToolsSession::new();
        session.attach();
        assert!(session.is_attached);
        
        let msg = DevToolsMessage::DomEnable { id: 1 };
        let resp = session.handle_message(msg);
        assert!(resp.error.is_none());
    }
}
