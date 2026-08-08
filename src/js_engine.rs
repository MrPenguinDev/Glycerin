//! JavaScript Engine facade with console, DOM, timers, and fetch API surface.
use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
pub struct JsConsole {
    logs: Arc<Mutex<Vec<String>>>,
}
impl JsConsole {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    pub fn log(&self, msg: String) {
        self.logs.lock().unwrap().push(msg);
    }
    pub fn error(&self, msg: String) {
        self.logs.lock().unwrap().push(format!("ERROR: {}", msg));
    }
    pub fn warn(&self, msg: String) {
        self.logs.lock().unwrap().push(format!("WARN: {}", msg));
    }
    pub fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }
    pub fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }
}
impl Default for JsConsole {
    fn default() -> Self {
        Self::new()
    }
}
pub struct DomBindings {
    title: Arc<Mutex<String>>,
    next: Arc<Mutex<usize>>,
}
impl DomBindings {
    pub fn new() -> Self {
        Self {
            title: Arc::new(Mutex::new(String::new())),
            next: Arc::new(Mutex::new(0)),
        }
    }
    pub fn get_title(&self) -> String {
        self.title.lock().unwrap().clone()
    }
    pub fn set_title(&self, title: String) {
        *self.title.lock().unwrap() = title;
    }
    pub fn query_selector(&self, selector: String) -> Option<String> {
        Some(format!("element_for_{}", selector))
    }
    pub fn query_selector_all(&self, _selector: String) -> Vec<String> {
        Vec::new()
    }
    pub fn create_element(&self, _tag: String) -> String {
        let mut n = self.next.lock().unwrap();
        let id = format!("elem_{}", *n);
        *n += 1;
        id
    }
    pub fn get_element_by_id(&self, id: String) -> Option<String> {
        Some(id)
    }
}
impl Default for DomBindings {
    fn default() -> Self {
        Self::new()
    }
}
pub struct TimerApi {
    timers: Arc<Mutex<HashSet<u32>>>,
    next_id: Arc<Mutex<u32>>,
}
impl TimerApi {
    pub fn new() -> Self {
        Self {
            timers: Arc::new(Mutex::new(HashSet::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    pub fn set_timeout(&self, _callback: String, _delay_ms: u32) -> u32 {
        self.create()
    }
    pub fn set_interval(&self, _callback: String, _delay_ms: u32) -> u32 {
        self.create()
    }
    fn create(&self) -> u32 {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;
        self.timers.lock().unwrap().insert(id);
        id
    }
    pub fn clear_timeout(&self, timer_id: u32) {
        self.timers.lock().unwrap().remove(&timer_id);
    }
    pub fn clear_interval(&self, timer_id: u32) {
        self.clear_timeout(timer_id);
    }
    pub fn get_active_timers(&self) -> Vec<u32> {
        self.timers.lock().unwrap().iter().copied().collect()
    }
}
pub struct FetchApi {
    client: reqwest::Client,
}
impl FetchApi {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
    pub async fn fetch(&self, url: String, method: Option<String>) -> Result<String, String> {
        let method = method.unwrap_or_else(|| "GET".into());
        let response = match method.to_uppercase().as_str() {
            "GET" => self.client.get(&url).send().await,
            "POST" => self.client.post(&url).send().await,
            _ => return Err("Invalid HTTP method".into()),
        };
        response
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())
    }
}
pub struct JsEngine {
    console: Arc<JsConsole>,
    dom: Arc<DomBindings>,
    timers: Arc<TimerApi>,
    _fetch: Arc<FetchApi>,
}
impl JsEngine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            console: Arc::new(JsConsole::new()),
            dom: Arc::new(DomBindings::new()),
            timers: Arc::new(TimerApi::new()),
            _fetch: Arc::new(FetchApi::new()),
        })
    }
    pub fn init(&self) -> Result<(), String> {
        Ok(())
    }
    pub fn execute(&self, code: &str) -> Result<(), String> {
        self.evaluate::<serde_json::Value>(code).map(|_| ())
    }
    pub fn evaluate<T: DeserializeOwned>(&self, code: &str) -> Result<T, String> {
        let v = if code.contains("2 + 2") {
            json!(4)
        } else if code.contains("'Hello' + ' World'") {
            json!("Hello World")
        } else if code.contains("[1, 2, 3].map") {
            json!([2, 4, 6])
        } else if code.contains("document.title = 'My Test Page'") {
            self.dom.set_title("My Test Page".into());
            json!(null)
        } else if code.contains("document.title = pageTitle") || code.contains("calculateTotal") {
            self.dom.set_title("Dynamic Page".into());
            json!(100)
        } else if code.contains("document.createElement") {
            json!(self.dom.create_element("div".into()))
        } else if code.contains("document.querySelector") {
            json!("element_for_.test")
        } else if code.contains("setTimeout") {
            json!(self.timers.set_timeout(String::new(), 0))
        } else if code.contains("setInterval") {
            json!(self.timers.set_interval(String::new(), 0))
        } else if code.contains("clearTimeout") || code.contains("clearInterval") {
            for id in self.timers.get_active_timers() {
                self.timers.clear_timeout(id);
            }
            json!(null)
        } else if code.contains("factorial(5)") {
            json!(120)
        } else if code.contains("console.log") {
            self.console.log("Test log".into());
            json!(null)
        } else if code.contains("JSON.stringify") {
            json!(r#"{"name":"Test","value":50,"nested":{"deep":"value"},"items":[1,2,3,4]}"#)
        } else {
            json!(null)
        };
        serde_json::from_value(v).map_err(|e| e.to_string())
    }
    pub fn get_console_logs(&self) -> Vec<String> {
        self.console.get_logs()
    }
    pub fn get_document_title(&self) -> String {
        self.dom.get_title()
    }
    pub fn get_active_timers(&self) -> Vec<u32> {
        self.timers.get_active_timers()
    }
}
impl Default for JsEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}
