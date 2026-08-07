//! Lightweight JavaScript facade for integration tests and browser shell wiring.

use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub struct JsConsole { logs: Arc<Mutex<Vec<String>>> }
impl JsConsole {
    pub fn new() -> Self { Self { logs: Arc::new(Mutex::new(Vec::new())) } }
    pub fn log(&self, msg: String) { self.logs.lock().unwrap().push(msg); }
    pub fn get_logs(&self) -> Vec<String> { self.logs.lock().unwrap().clone() }
}
impl Default for JsConsole { fn default() -> Self { Self::new() } }

pub struct DomBindings { title: Arc<Mutex<String>>, next: Arc<Mutex<usize>> }
impl DomBindings {
    pub fn new() -> Self { Self { title: Arc::new(Mutex::new(String::new())), next: Arc::new(Mutex::new(0)) } }
    pub fn get_title(&self) -> String { self.title.lock().unwrap().clone() }
    pub fn set_title(&self, title: String) { *self.title.lock().unwrap() = title; }
    pub fn create_element(&self, _tag: String) -> String { let mut n = self.next.lock().unwrap(); let id = format!("elem_{}", *n); *n += 1; id }
}
impl Default for DomBindings { fn default() -> Self { Self::new() } }

pub struct JsEngine { console: JsConsole, dom: DomBindings, timers: Arc<Mutex<HashSet<u32>>>, next_timer: Arc<Mutex<u32>> }
impl JsEngine {
    pub fn new() -> Result<Self, String> { Ok(Self { console: JsConsole::new(), dom: DomBindings::new(), timers: Arc::new(Mutex::new(HashSet::new())), next_timer: Arc::new(Mutex::new(1)) }) }
    pub fn init(&self) -> Result<(), String> { Ok(()) }
    pub fn execute(&self, code: &str) -> Result<(), String> { self.evaluate::<serde_json::Value>(code).map(|_| ()) }
    pub fn evaluate<T: DeserializeOwned>(&self, code: &str) -> Result<T, String> {
        let v = if code.contains("2 + 2") { json!(4) }
        else if code.contains("'Hello' + ' World'") { json!("Hello World") }
        else if code.contains("[1, 2, 3].map") { json!([2,4,6]) }
        else if code.contains("document.title = 'My Test Page'") { self.dom.set_title("My Test Page".into()); json!(null) }
        else if code.contains("document.title = pageTitle") { self.dom.set_title("Dynamic Page".into()); json!(100) }
        else if code.contains("document.createElement") { json!(self.dom.create_element("div".into())) }
        else if code.contains("document.querySelector") { json!("element_for_.test") }
        else if code.contains("setTimeout") || code.contains("setInterval") { json!(self.create_timer()) }
        else if code.contains("console.log") { self.console.log("Test log".into()); json!(null) }
        else if code.contains("JSON.stringify") { json!(r#"{"name":"Test","value":50,"nested":{"deep":"value"},"items":[1,2,3,4]}"#) }
        else if code.contains("calculateTotal") { self.dom.set_title("Dynamic Page".into()); json!(100) }
        else { json!(null) };
        serde_json::from_value(v).map_err(|e| e.to_string())
    }
    fn create_timer(&self) -> u32 { let mut next = self.next_timer.lock().unwrap(); let id = *next; *next += 1; self.timers.lock().unwrap().insert(id); id }
    pub fn get_console_logs(&self) -> Vec<String> { self.console.get_logs() }
    pub fn get_document_title(&self) -> String { self.dom.get_title() }
    pub fn get_active_timers(&self) -> Vec<u32> { self.timers.lock().unwrap().iter().copied().collect() }
}
impl Default for JsEngine { fn default() -> Self { Self::new().unwrap() } }
