//! Minimal browser chrome shell.

use iced::{Element, Subscription, Task};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BrowserTab { pub id: usize, pub title: String, pub url: String, pub can_go_back: bool, pub can_go_forward: bool, pub is_loading: bool }
impl BrowserTab { pub fn new(id: usize, url: String) -> Self { Self { id, title: "New Tab".into(), url, can_go_back: false, can_go_forward: false, is_loading: false } } }
#[derive(Debug, Clone)]
pub enum Message { UrlSubmitted(String), GoBack, GoForward, Reload, Stop, TabSelected(usize), NewTab, CloseTab(usize), UrlBarChanged(String), WindowResized(u32, u32) }
pub struct BrowserShell { tabs: HashMap<usize, BrowserTab>, active_tab_id: Option<usize>, next_tab_id: usize, url_bar_text: String }
impl BrowserShell {
    pub fn new() -> Self { let mut tabs = HashMap::new(); tabs.insert(0, BrowserTab::new(0, "about:blank".into())); Self { tabs, active_tab_id: Some(0), next_tab_id: 1, url_bar_text: String::new() } }
    pub fn update(&mut self, message: Message) -> Task<Message> { match message { Message::UrlSubmitted(url) => if let Some(tab) = self.active_tab_id.and_then(|id| self.tabs.get_mut(&id)) { tab.url = url.clone(); self.url_bar_text = url; }, Message::NewTab => { let id = self.next_tab_id; self.next_tab_id += 1; self.tabs.insert(id, BrowserTab::new(id, "about:blank".into())); self.active_tab_id = Some(id); }, Message::CloseTab(id) => { self.tabs.remove(&id); self.active_tab_id = self.tabs.keys().next().copied(); }, Message::TabSelected(id) => self.active_tab_id = Some(id), Message::UrlBarChanged(v) => self.url_bar_text = v, _ => {} } Task::none() }
    pub fn view(&self) -> Element<'_, Message> { iced::widget::text("Glycerin").into() }
    pub fn subscription(&self) -> Subscription<Message> { Subscription::none() }
    pub fn title(&self) -> String { "Glycerin Browser".into() }
}
impl Default for BrowserShell { fn default() -> Self { Self::new() } }
