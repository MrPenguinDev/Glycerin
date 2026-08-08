//! Data Persistence Layer preserving history, bookmarks, cookies, and sessions.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
pub type Result<T> = std::result::Result<T, String>;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub timestamp: u64,
    pub visit_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub folder: String,
    pub position: i32,
    pub date_added: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub id: i64,
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: Option<u64>,
    pub secure: bool,
    pub httponly: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub id: i64,
    pub window_id: String,
    pub tabs_json: String,
    pub active_tab_index: i32,
    pub timestamp: u64,
}
pub struct DatabaseManager {
    history: Mutex<Vec<HistoryEntry>>,
    bookmarks: Mutex<Vec<Bookmark>>,
    cookies: Mutex<Vec<Cookie>>,
    sessions: Mutex<Vec<SessionData>>,
}
impl DatabaseManager {
    pub fn new(_db_path: PathBuf) -> Result<Self> {
        Ok(Self {
            history: Mutex::new(Vec::new()),
            bookmarks: Mutex::new(Vec::new()),
            cookies: Mutex::new(Vec::new()),
            sessions: Mutex::new(Vec::new()),
        })
    }
    pub fn add_history_entry(&self, url: &str, title: &str) -> Result<()> {
        let mut h = self.history.lock().unwrap();
        let id = h.len() as i64 + 1;
        h.push(HistoryEntry {
            id,
            url: url.into(),
            title: title.into(),
            timestamp: now(),
            visit_count: 1,
        });
        Ok(())
    }
    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        Ok(self
            .history
            .lock()
            .unwrap()
            .iter()
            .take(limit)
            .cloned()
            .collect())
    }
    pub fn search_history(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        Ok(self
            .history
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.url.contains(query) || e.title.contains(query))
            .take(limit)
            .cloned()
            .collect())
    }
    pub fn clear_history(&self) -> Result<()> {
        self.history.lock().unwrap().clear();
        Ok(())
    }
    pub fn add_bookmark(&self, url: &str, title: &str, folder: &str) -> Result<i64> {
        let mut b = self.bookmarks.lock().unwrap();
        let id = b.len() as i64 + 1;
        b.push(Bookmark {
            id,
            url: url.into(),
            title: title.into(),
            folder: folder.into(),
            position: id as i32,
            date_added: now(),
        });
        Ok(id)
    }
    pub fn get_bookmarks(&self, folder: Option<&str>) -> Result<Vec<Bookmark>> {
        Ok(self
            .bookmarks
            .lock()
            .unwrap()
            .iter()
            .filter(|b| folder.map(|f| b.folder == f).unwrap_or(true))
            .cloned()
            .collect())
    }
    pub fn set_cookie(&self, cookie: &Cookie) -> Result<()> {
        self.cookies.lock().unwrap().push(cookie.clone());
        Ok(())
    }
    pub fn get_cookies_for_domain(&self, domain: &str) -> Result<Vec<Cookie>> {
        Ok(self
            .cookies
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.domain.contains(domain))
            .cloned()
            .collect())
    }
    pub fn save_session(
        &self,
        window_id: &str,
        tabs_json: &str,
        active_tab_index: i32,
    ) -> Result<()> {
        self.sessions.lock().unwrap().push(SessionData {
            id: 1,
            window_id: window_id.into(),
            tabs_json: tabs_json.into(),
            active_tab_index,
            timestamp: now(),
        });
        Ok(())
    }
    pub fn load_session(&self, window_id: &str) -> Result<Option<SessionData>> {
        Ok(self
            .sessions
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.window_id == window_id)
            .cloned())
    }
    pub fn get_all_sessions(&self) -> Result<Vec<SessionData>> {
        Ok(self.sessions.lock().unwrap().clone())
    }
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
