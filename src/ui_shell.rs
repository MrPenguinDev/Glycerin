//! Phase 1: Browser Chrome & UI Shell
//! Implements address bar, navigation buttons, tab management, and browser window

use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Length, Subscription, Theme};
use iced_aw::{TabLabel, Tabs};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BrowserTab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
}

impl BrowserTab {
    pub fn new(id: usize, url: String) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url,
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    UrlSubmitted(String),
    GoBack,
    GoForward,
    Reload,
    Stop,
    
    // Tab Management
    TabSelected(usize),
    NewTab,
    CloseTab(usize),
    
    // UI State
    UrlBarChanged(String),
    WindowResized(u32, u32),
}

pub struct BrowserShell {
    tabs: HashMap<usize, BrowserTab>,
    active_tab_id: Option<usize>,
    next_tab_id: usize,
    url_bar_text: String,
    window_width: u32,
    window_height: u32,
}

impl BrowserShell {
    pub fn new() -> Self {
        let mut tabs = HashMap::new();
        let first_tab = BrowserTab::new(0, "about:blank".to_string());
        tabs.insert(0, first_tab);
        
        Self {
            tabs,
            active_tab_id: Some(0),
            next_tab_id: 1,
            url_bar_text: String::new(),
            window_width: 1280,
            window_height: 720,
        }
    }
    
    pub fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::UrlSubmitted(url) => {
                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.url = url.clone();
                        tab.is_loading = true;
                        tab.can_go_back = true;
                    }
                }
            }
            
            Message::GoBack => {
                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.can_go_forward = true;
                        // In full implementation, navigate to previous history entry
                    }
                }
            }
            
            Message::GoForward => {
                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.can_go_back = true;
                        // In full implementation, navigate to next history entry
                    }
                }
            }
            
            Message::Reload => {
                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.is_loading = true;
                    }
                }
            }
            
            Message::TabSelected(tab_id) => {
                self.active_tab_id = Some(tab_id);
                if let Some(tab) = self.tabs.get(&tab_id) {
                    self.url_bar_text = tab.url.clone();
                }
            }
            
            Message::NewTab => {
                let new_tab = BrowserTab::new(self.next_tab_id, "about:blank".to_string());
                self.tabs.insert(self.next_tab_id, new_tab);
                self.active_tab_id = Some(self.next_tab_id);
                self.next_tab_id += 1;
            }
            
            Message::CloseTab(tab_id) => {
                self.tabs.remove(&tab_id);
                
                if self.tabs.is_empty() {
                    // Create a new tab if all tabs are closed
                    let new_tab = BrowserTab::new(self.next_tab_id, "about:blank".to_string());
                    self.tabs.insert(self.next_tab_id, new_tab);
                    self.active_tab_id = Some(self.next_tab_id);
                    self.next_tab_id += 1;
                } else if let Some(current_id) = self.active_tab_id {
                    if current_id == tab_id {
                        // Select another tab
                        self.active_tab_id = self.tabs.keys().next().copied();
                    }
                }
            }
            
            Message::UrlBarChanged(text) => {
                self.url_bar_text = text;
            }
            
            Message::WindowResized(width, height) => {
                self.window_width = width;
                self.window_height = height;
            }
            
            _ => {}
        }
        
        iced::Task::none()
    }
    
    pub fn view(&self) -> Element<Message> {
        // Build tab bar
        let mut tab_labels = Vec::new();
        for (id, tab) in &self.tabs {
            let label = if tab.title.is_empty() { 
                "New Tab".to_string() 
            } else { 
                tab.title.clone() 
            };
            
            let close_btn = button(text("×"))
                .on_press(Message::CloseTab(*id));
            
            let tab_content = row![text(label), close_btn]
                .spacing(5);
            
            tab_labels.push((
                TabLabel::Text(tab_content.into()),
                *id,
            ));
        }
        
        let tabs_widget = Tabs::new(
            self.active_tab_id,
            tab_labels.into_iter().map(|(label, id)| {
                (label, Message::TabSelected(id))
            }).collect(),
        )
        .on_close(Message::CloseTab);
        
        // Build navigation bar
        let back_btn = button(text("←"))
            .on_press(Message::GoBack);
        
        let forward_btn = button(text("→"))
            .on_press(Message::GoForward);
        
        let reload_btn = button(text("↻"))
            .on_press(Message::Reload);
        
        let url_input = text_input("Enter URL...", &self.url_bar_text)
            .on_submit(Message::UrlSubmitted(self.url_bar_text.clone()))
            .on_input(Message::UrlBarChanged)
            .width(Length::Fill);
        
        let nav_bar = row![
            back_btn,
            forward_btn,
            reload_btn,
            url_input.width(Length::Fill),
            button(text("+")).on_press(Message::NewTab)
        ]
        .spacing(5)
        .padding(5);
        
        // Main layout
        let content = column![
            nav_bar,
            tabs_widget,
            container(text("Browser Content Area"))
                .width(Length::Fill)
                .height(Length::Fill)
        ]
        .spacing(5)
        .padding(5);
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

impl Default for BrowserShell {
    fn default() -> Self {
        Self::new()
    }
}
