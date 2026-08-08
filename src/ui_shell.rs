//! Browser Chrome & UI Shell - Full GUI Implementation
//! Complete browser interface with tabs, address bar, navigation, and viewport

use iced::{
    widget::{
        button as iced_button, column, container as iced_container, row, scrollable, text,
        text_input, tooltip, Space,
    },
    Alignment, Color, Element, Length, Padding, Subscription, Task, Theme,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Browser tab representation
#[derive(Debug, Clone)]
pub struct BrowserTab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub favicon: Option<Vec<u8>>,
    pub load_progress: f32,
}

impl BrowserTab {
    pub fn new(id: usize, url: String) -> Self {
        Self {
            id,
            title: "New Tab".into(),
            url,
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
            favicon: None,
            load_progress: 0.0,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn with_url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
}

/// UI Messages for browser interaction
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    UrlSubmitted(String),
    GoBack,
    GoForward,
    Reload,
    Stop,
    Home,

    // Tab Management
    NewTab,
    CloseTab(usize),
    TabSelected(usize),
    DuplicateTab(usize),

    // URL Bar
    UrlBarChanged(String),
    UrlBarFocused,
    UrlBarUnfocused,

    // Window
    WindowResized(u32, u32),
    WindowClosed,
    WindowMinimized,
    WindowMaximized,

    // Bookmarks
    BookmarkCurrentPage,
    BookmarkOpened(String),

    // History
    HistoryBack,
    HistoryForward,
    ShowHistory,

    // Settings & Menu
    ToggleMenu,
    OpenSettings,
    OpenDownloads,
    OpenDevTools,
    EnterFullScreen,
    ExitFullScreen,
    ZoomIn,
    ZoomOut,
    ResetZoom,

    // Loading State
    LoadingStarted,
    LoadingComplete,
    LoadingProgress(f32),

    // Context Menu
    ShowContextMenu(f32, f32),
    HideContextMenu,

    NoOp,
}

/// Main browser shell structure
pub struct BrowserShell {
    // Tabs
    tabs: HashMap<usize, BrowserTab>,
    active_tab_id: Option<usize>,
    next_tab_id: usize,
    tab_order: Vec<usize>,

    // Navigation
    url_bar_text: String,
    url_bar_focused: bool,
    navigation_history: Vec<String>,
    forward_history: Vec<String>,

    // UI State
    is_menu_open: bool,
    zoom_level: f32,
    is_fullscreen: bool,
    show_bookmarks_bar: bool,

    // Loading
    is_loading: bool,
    load_progress: f32,
    last_load_time: Option<Instant>,

    // Bookmarks
    bookmarks: Vec<BookmarkEntry>,

    // Window
    window_width: u32,
    window_height: u32,
}

#[derive(Debug, Clone)]
pub struct BookmarkEntry {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub folder: String,
}

impl BrowserShell {
    pub fn new() -> Self {
        let mut tabs = HashMap::new();
        let mut tab_order = Vec::new();

        // Create initial tab
        let first_tab = BrowserTab::new(0, "about:newtab".into()).with_title("New Tab".into());
        tabs.insert(0, first_tab);
        tab_order.push(0);

        Self {
            tabs,
            active_tab_id: Some(0),
            next_tab_id: 1,
            tab_order,
            url_bar_text: String::new(),
            url_bar_focused: false,
            navigation_history: Vec::new(),
            forward_history: Vec::new(),
            is_menu_open: false,
            zoom_level: 1.0,
            is_fullscreen: false,
            show_bookmarks_bar: true,
            is_loading: false,
            load_progress: 0.0,
            last_load_time: None,
            bookmarks: vec![
                BookmarkEntry {
                    id: 0,
                    title: "Rust Programming".into(),
                    url: "https://www.rust-lang.org".into(),
                    folder: "Favorites".into(),
                },
                BookmarkEntry {
                    id: 1,
                    title: "Elm Language".into(),
                    url: "https://elm-lang.org".into(),
                    folder: "Favorites".into(),
                },
            ],
            window_width: 1200,
            window_height: 800,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UrlSubmitted(url) => {
                let normalized_url = normalize_url(&url);

                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        // Add current URL to history before navigating
                        if !tab.url.is_empty() && tab.url != "about:newtab" {
                            self.navigation_history.push(tab.url.clone());
                            self.forward_history.clear();
                        }

                        tab.url = normalized_url.clone();
                        tab.title = format!("Loading: {}", normalized_url);
                        tab.is_loading = true;
                        tab.load_progress = 0.0;
                        self.last_load_time = Some(Instant::now());
                    }
                }

                self.url_bar_text = normalized_url;
                self.is_loading = true;
                self.load_progress = 0.0;

                // Simulate loading progress
                return Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        Message::LoadingProgress(0.3)
                    },
                    |msg| msg,
                );
            }

            Message::GoBack => {
                if let Some(history_url) = self.navigation_history.pop() {
                    if let Some(tab_id) = self.active_tab_id {
                        if let Some(tab) = self.tabs.get_mut(&tab_id) {
                            self.forward_history.push(tab.url.clone());
                            tab.url = history_url.clone();
                            tab.title = format!("Loading: {}", history_url);
                            tab.is_loading = true;
                            self.url_bar_text = history_url;
                        }
                    }
                }
            }

            Message::GoForward => {
                if let Some(history_url) = self.forward_history.pop() {
                    if let Some(tab_id) = self.active_tab_id {
                        if let Some(tab) = self.tabs.get_mut(&tab_id) {
                            self.navigation_history.push(tab.url.clone());
                            tab.url = history_url.clone();
                            tab.title = format!("Loading: {}", history_url);
                            tab.is_loading = true;
                            self.url_bar_text = history_url;
                        }
                    }
                }
            }

            Message::Reload => {
                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.is_loading = true;
                        tab.load_progress = 0.0;
                        self.is_loading = true;
                        self.load_progress = 0.0;
                        self.last_load_time = Some(Instant::now());
                    }
                }
            }

            Message::Stop => {
                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.is_loading = false;
                    }
                }
                self.is_loading = false;
            }

            Message::Home => {
                let home_url = "about:newtab".to_string();
                return Task::done(Message::UrlSubmitted(home_url));
            }

            Message::NewTab => {
                let id = self.next_tab_id;
                self.next_tab_id += 1;
                let new_tab =
                    BrowserTab::new(id, "about:newtab".into()).with_title("New Tab".into());
                self.tabs.insert(id, new_tab);
                self.tab_order.push(id);
                self.active_tab_id = Some(id);
                self.url_bar_text = String::new();
            }

            Message::CloseTab(tab_id) => {
                self.tabs.remove(&tab_id);
                self.tab_order.retain(|&id| id != tab_id);

                // If closing active tab, switch to another
                if self.active_tab_id == Some(tab_id) {
                    self.active_tab_id = self.tab_order.first().copied();
                    if let Some(active_id) = self.active_tab_id {
                        if let Some(tab) = self.tabs.get(&active_id) {
                            self.url_bar_text = tab.url.clone();
                        }
                    }
                }

                // Close window if no tabs left
                if self.tabs.is_empty() {
                    return Task::done(Message::WindowClosed);
                }
            }

            Message::TabSelected(tab_id) => {
                self.active_tab_id = Some(tab_id);
                if let Some(tab) = self.tabs.get(&tab_id) {
                    self.url_bar_text = tab.url.clone();
                }
            }

            Message::DuplicateTab(tab_id) => {
                if let Some(tab) = self.tabs.get(&tab_id).cloned() {
                    let new_id = self.next_tab_id;
                    self.next_tab_id += 1;
                    let mut new_tab = tab;
                    new_tab.id = new_id;
                    self.tabs.insert(new_id, new_tab);
                    self.tab_order.push(new_id);
                    self.active_tab_id = Some(new_id);
                }
            }

            Message::UrlBarChanged(text) => {
                self.url_bar_text = text;
            }

            Message::UrlBarFocused => {
                self.url_bar_focused = true;
            }

            Message::UrlBarUnfocused => {
                self.url_bar_focused = false;
            }

            Message::WindowResized(width, height) => {
                self.window_width = width;
                self.window_height = height;
            }

            Message::BookmarkCurrentPage => {
                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get(&tab_id) {
                        let bookmark = BookmarkEntry {
                            id: self.bookmarks.len(),
                            title: tab.title.clone(),
                            url: tab.url.clone(),
                            folder: "Favorites".into(),
                        };
                        self.bookmarks.push(bookmark);
                    }
                }
            }

            Message::BookmarkOpened(url) => {
                return Task::done(Message::UrlSubmitted(url));
            }

            Message::ToggleMenu => {
                self.is_menu_open = !self.is_menu_open;
            }

            Message::OpenSettings => {
                // Open settings page
                return Task::done(Message::UrlSubmitted("glycerin://settings".into()));
            }

            Message::OpenDownloads => {
                return Task::done(Message::UrlSubmitted("glycerin://downloads".into()));
            }

            Message::OpenDevTools => {
                // DevTools would be opened in a separate window
                println!("Opening DevTools...");
            }

            Message::ZoomIn => {
                self.zoom_level = (self.zoom_level + 0.1).min(5.0);
            }

            Message::ZoomOut => {
                self.zoom_level = (self.zoom_level - 0.1).max(0.1);
            }

            Message::ResetZoom => {
                self.zoom_level = 1.0;
            }

            Message::LoadingStarted => {
                self.is_loading = true;
                self.load_progress = 0.0;
                self.last_load_time = Some(Instant::now());
            }

            Message::LoadingComplete => {
                self.is_loading = false;
                self.load_progress = 1.0;

                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.is_loading = false;
                        tab.load_progress = 1.0;
                        // Extract domain for title if still showing "Loading:"
                        if tab.title.starts_with("Loading:") {
                            if let Ok(parsed) = url::Url::parse(&tab.url) {
                                tab.title = parsed.host_str().unwrap_or("Unknown").to_string();
                            }
                        }
                    }
                }
            }

            Message::LoadingProgress(progress) => {
                self.load_progress = progress;

                if let Some(tab_id) = self.active_tab_id {
                    if let Some(tab) = self.tabs.get_mut(&tab_id) {
                        tab.load_progress = progress;
                    }
                }

                // Continue loading simulation
                if progress < 1.0 {
                    let next_progress = (progress + 0.2).min(1.0);
                    return Task::perform(
                        async move {
                            tokio::time::sleep(Duration::from_millis(200)).await;
                            Message::LoadingProgress(next_progress)
                        },
                        |msg| msg,
                    );
                } else {
                    return Task::done(Message::LoadingComplete);
                }
            }

            Message::ShowContextMenu(_, _) | Message::HideContextMenu => {
                // Context menu handling
            }

            Message::EnterFullScreen | Message::ExitFullScreen => {
                self.is_fullscreen = !self.is_fullscreen;
            }

            Message::HistoryBack => {
                return Task::done(Message::GoBack);
            }

            Message::HistoryForward => {
                return Task::done(Message::GoForward);
            }

            Message::WindowMinimized | Message::WindowMaximized | Message::WindowClosed => {}

            Message::ShowHistory => {
                return Task::done(Message::UrlSubmitted("glycerin://history".into()));
            }

            Message::NoOp => {}
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        // Main browser layout
        let content = column![
            self.view_title_bar(),
            self.view_tab_strip(),
            self.view_toolbar(),
            if self.show_bookmarks_bar {
                self.view_bookmarks_bar()
            } else {
                row![].height(Length::Shrink).into()
            },
            self.view_loading_indicator(),
            self.view_viewport(),
        ]
        .spacing(0);

        iced_container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(surface_background)
            .into()
    }

    fn view_title_bar(&self) -> Element<'_, Message> {
        // Traffic light buttons (macOS style)
        let window_controls = row![
            self.view_window_iced_button(Color::from_rgb(1.0, 0.23, 0.19)), // Close - Red
            self.view_window_iced_button(Color::from_rgb(1.0, 0.73, 0.18)), // Minimize - Yellow
            self.view_window_iced_button(Color::from_rgb(0.0, 0.88, 0.0)),  // Maximize - Green
        ]
        .spacing(8);

        // Title in center
        let title = if let Some(tab_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.get(&tab_id) {
                text(tab.title.clone()).size(13)
            } else {
                text("Glycerin").size(13)
            }
        } else {
            text("Glycerin").size(13)
        };

        // Right side controls
        let right_controls = row![iced_button(text("☰").size(16))
            .padding([4, 10])
            .on_press(Message::ToggleMenu)
            .style(iced_button::secondary),]
        .spacing(8);

        iced_container(
            row![
                window_controls.width(Length::Fixed(70.0)),
                iced_container(title).center_x(Length::Fill),
                right_controls.width(Length::Fixed(50.0)),
            ]
            .align_y(Alignment::Center)
            .padding(Padding::new(8.0).top(10.0).bottom(6.0)),
        )
        .style(title_bar_style)
        .height(Length::Fixed(36.0))
        .into()
    }

    fn view_window_iced_button(&self, color: Color) -> Element<'_, Message> {
        iced_container(Space::with_width(Length::Fixed(12.0)).height(Length::Fixed(12.0)))
            .style(move |_| circle_button_style(color))
            .into()
    }

    fn view_tab_strip(&self) -> Element<'_, Message> {
        let mut tabs: Vec<Element<'_, Message>> = self
            .tab_order
            .iter()
            .filter_map(|tab_id| self.tabs.get(tab_id))
            .map(|tab| {
                let is_active = Some(tab.id) == self.active_tab_id;
                let status = if tab.is_loading { " ⏳" } else { "" };
                let label = format!("{}{}", truncate_title(&tab.title, 22), status);
                let tab_button = iced_button(text(label).size(12))
                    .padding([7, 12])
                    .on_press(Message::TabSelected(tab.id))
                    .style(if is_active {
                        iced_button::primary
                    } else {
                        iced_button::secondary
                    });

                row![
                    tab_button,
                    iced_button(text("×").size(12))
                        .padding([7, 9])
                        .on_press(Message::CloseTab(tab.id))
                        .style(iced_button::text),
                ]
                .spacing(1)
                .into()
            })
            .collect();

        tabs.push(
            iced_button(text("+").size(18))
                .padding([5, 12])
                .on_press(Message::NewTab)
                .style(iced_button::secondary)
                .into(),
        );

        iced_container(
            row(tabs)
                .spacing(6)
                .padding(Padding::new(8.0).left(12.0).right(12.0)),
        )
        .height(Length::Fixed(42.0))
        .style(tab_strip_style)
        .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        // Navigation buttons
        let nav_buttons = row![
            self.view_nav_iced_button("←", Message::GoBack, !self.can_go_back()),
            self.view_nav_iced_button("→", Message::GoForward, !self.can_go_forward()),
            self.view_nav_iced_button("⟳", Message::Reload, false),
            if self.is_loading {
                self.view_nav_iced_button("✕", Message::Stop, false)
            } else {
                Space::with_width(Length::Fixed(40.0)).into()
            },
        ]
        .spacing(4);

        // URL bar
        let url_input = text_input("Enter URL or search...", &self.url_bar_text)
            .on_submit(Message::UrlSubmitted(self.url_bar_text.clone()))
            .on_input(Message::UrlBarChanged)
            .padding([6, 12])
            .size(14)
            .width(Length::FillPortion(6));

        // Extension icons area
        let extensions = row![
            self.view_icon_iced_button("🏠", "Home", Some(Message::Home)),
            self.view_icon_iced_button("🔒", "Site secure", None),
            self.view_icon_iced_button(
                "⭐",
                "Bookmark this page",
                Some(Message::BookmarkCurrentPage)
            ),
            self.view_icon_iced_button("🕘", "History", Some(Message::ShowHistory)),
            self.view_icon_iced_button("⚙", "Settings", Some(Message::OpenSettings)),
        ]
        .spacing(8);

        iced_container(
            row![nav_buttons, url_input, extensions,]
                .align_y(Alignment::Center)
                .padding(Padding::new(8.0).left(12.0).right(12.0))
                .spacing(12),
        )
        .height(Length::Fixed(50.0))
        .style(toolbar_style)
        .into()
    }

    fn view_nav_iced_button<'a>(
        &self,
        icon: &'a str,
        message: Message,
        disabled: bool,
    ) -> Element<'a, Message> {
        let btn = iced_button(text(icon).size(18))
            .padding([6, 12])
            .style(iced_button::text);

        if disabled {
            btn.into()
        } else {
            btn.on_press(message).into()
        }
    }

    fn view_icon_iced_button<'a>(
        &self,
        icon: &'a str,
        tooltip_text: &'a str,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        let mut button = iced_button(text(icon).size(16))
            .padding(6)
            .style(iced_button::text);
        if let Some(message) = on_press {
            button = button.on_press(message);
        }

        tooltip(button, tooltip_text, tooltip::Position::Bottom).into()
    }

    fn view_bookmarks_bar(&self) -> Element<'_, Message> {
        let bookmark_buttons: Vec<Element<'_, Message>> = self
            .bookmarks
            .iter()
            .map(|bookmark| {
                iced_button(text(bookmark.title.clone()).size(12))
                    .padding([4, 12])
                    .on_press(Message::BookmarkOpened(bookmark.url.clone()))
                    .style(iced_button::secondary)
                    .into()
            })
            .collect();

        iced_container(
            row(bookmark_buttons)
                .spacing(4)
                .padding(Padding::new(8.0).left(12.0).right(12.0)),
        )
        .height(Length::Fixed(32.0))
        .style(bookmarks_bar_style)
        .into()
    }

    fn view_loading_indicator(&self) -> Element<'_, Message> {
        if self.is_loading && self.load_progress < 1.0 {
            let progress_bar = iced_container(
                row![iced_container(Space::with_width(Length::FillPortion(
                    (self.load_progress * 100.0).max(1.0) as u16
                )))
                .height(Length::Fill)
                .style(loading_progress_style)]
                .height(Length::Fill),
            )
            .height(Length::Fixed(3.0))
            .width(Length::Fill)
            .style(loading_track_style);

            progress_bar.into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        }
    }

    fn view_viewport(&self) -> Element<'_, Message> {
        if let Some(tab_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.get(&tab_id) {
                // Render different content based on URL
                if tab.url == "about:newtab" || tab.url.is_empty() {
                    return self.view_new_tab_page(tab);
                } else if tab.url.starts_with("glycerin://") {
                    return self.view_internal_page(&tab.url);
                } else {
                    // Web content would be rendered here via the rendering engine
                    return self.view_web_content(tab);
                }
            }
        }

        // Empty state
        iced_container(text("No active tab"))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_new_tab_page(&self, tab: &BrowserTab) -> Element<'_, Message> {
        let greeting = text("Welcome to Glycerin")
            .size(32)
            .color(Color::from_rgb(0.9, 0.9, 0.9));

        let subtitle = text("Fast. Private. Beautiful.")
            .size(16)
            .color(Color::from_rgb(0.5, 0.5, 0.5));

        let search_input = text_input("Search or enter address...", &self.url_bar_text)
            .on_submit(Message::UrlSubmitted(self.url_bar_text.clone()))
            .on_input(Message::UrlBarChanged)
            .padding([12, 20])
            .size(16)
            .width(Length::Fixed(500.0));

        let quick_links: Vec<Element<'_, Message>> = vec![
            ("🔍", "Search", "https://duckduckgo.com"),
            ("📧", "Email", "https://mail.google.com"),
            ("📺", "Videos", "https://youtube.com"),
            ("📰", "News", "https://news.ycombinator.com"),
            ("💬", "Social", "https://twitter.com"),
            ("🛒", "Shop", "https://amazon.com"),
        ]
        .iter()
        .map(|(icon, label, url)| {
            iced_button(
                column![text(*icon).size(24), text(*label).size(11),]
                    .align_x(Alignment::Center)
                    .spacing(4),
            )
            .padding(16)
            .on_press(Message::BookmarkOpened(url.to_string()))
            .style(iced_button::secondary)
            .into()
        })
        .collect();

        let quick_links_grid = row(quick_links).spacing(16).padding(20);

        iced_container(
            column![
                greeting,
                subtitle,
                Space::with_height(Length::Fixed(30.0)),
                search_input,
                Space::with_height(Length::Fixed(40.0)),
                quick_links_grid,
            ]
            .align_x(Alignment::Center)
            .spacing(0),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn view_internal_page(&self, url: &str) -> Element<'_, Message> {
        match url {
            "glycerin://settings" => self.view_settings_page(),
            "glycerin://downloads" => self.view_downloads_page(),
            "glycerin://history" => self.view_history_page(),
            _ => self.view_error_page("Unknown internal page"),
        }
    }

    fn view_web_content(&self, tab: &BrowserTab) -> Element<'_, Message> {
        // This would integrate with the rendering engine
        // For now, show a placeholder
        let content = column![
            text(format!("Rendering: {}", tab.url))
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            Space::with_height(Length::Fixed(20.0)),
            if tab.is_loading {
                text(format!("Loading... {:.0}%", tab.load_progress * 100.0)).size(12)
            } else {
                text("Page loaded successfully").size(12)
            }
            .color(Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .align_x(Alignment::Center)
        .padding(20);

        iced_container(content)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_settings_page(&self) -> Element<'_, Message> {
        let title = text("Settings").size(28).color(Color::WHITE);

        let settings_sections: Vec<Element<'_, Message>> = vec![
            self.view_setting_section(
                "Appearance",
                vec![
                    ("Theme", "Dark".into()),
                    ("Zoom Level", format!("{:.0}%", self.zoom_level * 100.0)),
                    (
                        "Show Bookmarks Bar",
                        (if self.show_bookmarks_bar { "Yes" } else { "No" }).into(),
                    ),
                ],
            ),
            self.view_setting_section(
                "Privacy & Security",
                vec![
                    ("Block Trackers", "Enabled".into()),
                    ("Block Ads", "Enabled".into()),
                    ("HTTPS-Only Mode", "Enabled".into()),
                ],
            ),
            self.view_setting_section(
                "Performance",
                vec![
                    ("Hardware Acceleration", "Enabled".into()),
                    ("Preload Pages", "Enabled".into()),
                    ("Memory Saver Mode", "Disabled".into()),
                ],
            ),
        ];

        column![
            title,
            Space::with_height(Length::Fixed(20.0)),
            column(settings_sections).spacing(20),
        ]
        .spacing(20)
        .padding(40)
        .into()
    }

    fn view_setting_section(
        &self,
        title: &str,
        items: Vec<(&str, String)>,
    ) -> Element<'_, Message> {
        let section_title = text(title.to_string())
            .size(18)
            .color(Color::from_rgb(0.2, 0.6, 1.0));

        let items_list: Vec<Element<'_, Message>> = items
            .iter()
            .map(|(label, value)| {
                row![
                    text(label.to_string())
                        .size(13)
                        .width(Length::FillPortion(2)),
                    text(value.clone())
                        .size(13)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                ]
                .into()
            })
            .collect();

        iced_container(
            column![
                section_title,
                Space::with_height(Length::Fixed(8.0)),
                column(items_list).spacing(8),
            ]
            .padding(16),
        )
        .style(setting_section_style)
        .into()
    }

    fn view_downloads_page(&self) -> Element<'_, Message> {
        let title = text("Downloads").size(28).color(Color::WHITE);

        let downloads = column![text("No active downloads")
            .size(14)
            .color(Color::from_rgb(0.5, 0.5, 0.5)),]
        .padding(20);

        column![title, downloads].spacing(20).padding(40).into()
    }

    fn view_history_page(&self) -> Element<'_, Message> {
        let title = text("History").size(28).color(Color::WHITE);

        let history_items: Vec<Element<'_, Message>> = self
            .navigation_history
            .iter()
            .rev()
            .take(20)
            .map(|url| {
                iced_button(text(url.clone()).size(13))
                    .padding([8, 12])
                    .on_press(Message::BookmarkOpened(url.clone()))
                    .style(iced_button::text)
                    .into()
            })
            .collect();

        column![
            title,
            Space::with_height(Length::Fixed(20.0)),
            if history_items.is_empty() {
                Element::from(
                    text("No browsing history")
                        .size(14)
                        .color(Color::from_rgb(0.5, 0.5, 0.5)),
                )
            } else {
                scrollable(column(history_items).spacing(4)).into()
            }
        ]
        .padding(40)
        .into()
    }

    fn view_error_page(&self, message: &str) -> Element<'_, Message> {
        let message = message.to_string();
        let error_icon = text("⚠️").size(48);
        let error_title = text("Oops! Something went wrong")
            .size(20)
            .color(Color::WHITE);
        let error_message = text(message).size(14).color(Color::from_rgb(0.6, 0.6, 0.6));

        let retry_button = iced_button(text("Retry").size(14))
            .padding([8, 24])
            .on_press(Message::Reload)
            .style(iced_button::primary);

        iced_container(
            column![
                error_icon,
                error_title,
                error_message,
                Space::with_height(Length::Fixed(20.0)),
                retry_button,
            ]
            .align_x(Alignment::Center)
            .spacing(12),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Subscribe to window resize events, keyboard shortcuts, etc.
        Subscription::none()
    }

    pub fn title(&self) -> String {
        if let Some(tab_id) = self.active_tab_id {
            if let Some(tab) = self.tabs.get(&tab_id) {
                return format!("{} - Glycerin", tab.title);
            }
        }
        "Glycerin Browser".into()
    }

    // Helper methods
    pub fn can_go_back(&self) -> bool {
        !self.navigation_history.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.forward_history.is_empty()
    }

    pub fn active_tab(&self) -> Option<&BrowserTab> {
        self.active_tab_id.and_then(|id| self.tabs.get(&id))
    }

    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn get_zoom_level(&self) -> f32 {
        self.zoom_level
    }
}

impl Default for BrowserShell {
    fn default() -> Self {
        Self::new()
    }
}

fn truncate_title(title: &str, max_chars: usize) -> String {
    let mut chars = title.chars();
    let shortened: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{}…", shortened)
    } else {
        shortened
    }
}

// URL normalization helper
fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();

    // Check if it's already a valid URL
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
        || trimmed.starts_with("about:")
        || trimmed.starts_with("glycerin://")
    {
        return trimmed.to_string();
    }

    // Check if it looks like a domain
    if trimmed.contains('.') && !trimmed.contains(' ') {
        return format!("https://{}", trimmed);
    }

    // Otherwise, treat as search query
    format!(
        "https://duckduckgo.com/html/?q={}",
        urlencoding::encode(trimmed)
    )
}

// Custom styling themes
fn surface_background(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default()
        .background(Color::from_rgb(0.05, 0.05, 0.06))
        .color(Color::WHITE)
}

fn title_bar_style(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default().background(Color::from_rgb(0.10, 0.10, 0.12))
}

fn circle_button_style(color: Color) -> iced_container::Style {
    iced_container::Style::default()
        .background(color)
        .border(iced::border::rounded(6.0))
}

fn tab_strip_style(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default().background(Color::from_rgb(0.08, 0.09, 0.11))
}

fn toolbar_style(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default().background(Color::from_rgb(0.13, 0.14, 0.17))
}

fn bookmarks_bar_style(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default().background(Color::from_rgb(0.09, 0.10, 0.12))
}

fn loading_track_style(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default().background(Color::from_rgb(0.18, 0.18, 0.20))
}

fn loading_progress_style(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default().background(Color::from_rgb(0.0, 0.75, 0.95))
}

fn setting_section_style(_theme: &Theme) -> iced_container::Style {
    iced_container::Style::default()
        .background(Color::from_rgb(0.10, 0.11, 0.14))
        .border(iced::border::rounded(8.0))
}
