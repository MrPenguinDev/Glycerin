//! Phase 2: Data Persistence Layer
//! Implements SQLite database for history, bookmarks, cookies, and session management

use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
    conn: Connection,
}

impl DatabaseManager {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&db_path)?;
        
        let mut mgr = Self { conn };
        mgr.initialize_schema()?;
        Ok(mgr)
    }
    
    fn initialize_schema(&mut self) -> Result<()> {
        // History table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                visit_count INTEGER DEFAULT 1
            )",
            [],
        )?;
        
        // Bookmarks table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                title TEXT NOT NULL,
                folder TEXT DEFAULT 'Other Bookmarks',
                position INTEGER DEFAULT 0,
                date_added INTEGER NOT NULL
            )",
            [],
        )?;
        
        // Cookies table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS cookies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                value TEXT NOT NULL,
                domain TEXT NOT NULL,
                path TEXT DEFAULT '/',
                expires INTEGER,
                secure BOOLEAN DEFAULT 0,
                httponly BOOLEAN DEFAULT 0,
                UNIQUE(name, domain, path)
            )",
            [],
        )?;
        
        // Sessions table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                window_id TEXT NOT NULL,
                tabs_json TEXT NOT NULL,
                active_tab_index INTEGER DEFAULT 0,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        
        // Create indexes for performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_url ON history(url)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_folder ON bookmarks(folder)",
            [],
        )?;
        
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cookies_domain ON cookies(domain)",
            [],
        )?;
        
        Ok(())
    }
    
    // History operations
    pub fn add_history_entry(&self, url: &str, title: &str) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Check if URL exists
        let mut stmt = self.conn.prepare(
            "SELECT id, visit_count FROM history WHERE url = ?"
        )?;
        
        if let Ok(mut rows) = stmt.query([url]) {
            if let Some(row) = rows.next()? {
                let id: i64 = row.get(0)?;
                let count: u32 = row.get(1)?;
                
                self.conn.execute(
                    "UPDATE history SET visit_count = ?, timestamp = ? WHERE id = ?",
                    [count + 1, timestamp, id],
                )?;
                return Ok(());
            }
        }
        
        // Insert new entry
        self.conn.execute(
            "INSERT INTO history (url, title, timestamp, visit_count) VALUES (?, ?, ?, 1)",
            [url, title, timestamp],
        )?;
        
        Ok(())
    }
    
    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, timestamp, visit_count 
             FROM history 
             ORDER BY timestamp DESC 
             LIMIT ?"
        )?;
        
        let entries = stmt.query_map([limit as u32], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                timestamp: row.get(3)?,
                visit_count: row.get(4)?,
            })
        })?;
        
        entries.collect()
    }
    
    pub fn search_history(&self, query: &str, limit: usize) -> Result<Vec<HistoryEntry>> {
        let search_pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id, url, title, timestamp, visit_count 
             FROM history 
             WHERE url LIKE ? OR title LIKE ?
             ORDER BY timestamp DESC 
             LIMIT ?"
        )?;
        
        let entries = stmt.query_map([&search_pattern, &search_pattern, limit as u32], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                timestamp: row.get(3)?,
                visit_count: row.get(4)?,
            })
        })?;
        
        entries.collect()
    }
    
    pub fn clear_history(&self) -> Result<()> {
        self.conn.execute("DELETE FROM history", [])?;
        Ok(())
    }
    
    // Bookmark operations
    pub fn add_bookmark(&self, url: &str, title: &str, folder: &str) -> Result<i64> {
        let date_added = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Get max position in folder
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(MAX(position), -1) FROM bookmarks WHERE folder = ?"
        )?;
        let position: i32 = stmt.query_row([folder], |row| row.get(0))?;
        
        self.conn.execute(
            "INSERT INTO bookmarks (url, title, folder, position, date_added) 
             VALUES (?, ?, ?, ?, ?)",
            [url, title, folder, position + 1, date_added],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn get_bookmarks(&self, folder: Option<&str>) -> Result<Vec<Bookmark>> {
        let sql = match folder {
            Some(f) => "SELECT id, url, title, folder, position, date_added 
                        FROM bookmarks 
                        WHERE folder = ? 
                        ORDER BY position".to_string(),
            None => "SELECT id, url, title, folder, position, date_added 
                     FROM bookmarks 
                     ORDER BY folder, position".to_string(),
        };
        
        let mut stmt = self.conn.prepare(&sql)?;
        
        let bookmarks = match folder {
            Some(f) => stmt.query_map([f], |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    folder: row.get(3)?,
                    position: row.get(4)?,
                    date_added: row.get(5)?,
                })
            })?,
            None => stmt.query_map([], |row| {
                Ok(Bookmark {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    folder: row.get(3)?,
                    position: row.get(4)?,
                    date_added: row.get(5)?,
                })
            })?,
        };
        
        bookmarks.collect()
    }
    
    pub fn remove_bookmark(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM bookmarks WHERE id = ?", [id])?;
        Ok(())
    }
    
    // Cookie operations
    pub fn set_cookie(&self, cookie: &Cookie) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO cookies 
             (name, value, domain, path, expires, secure, httponly) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            [
                &cookie.name,
                &cookie.value,
                &cookie.domain,
                &cookie.path,
                cookie.expires,
                if cookie.secure { 1 } else { 0 },
                if cookie.httponly { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }
    
    pub fn get_cookies_for_domain(&self, domain: &str) -> Result<Vec<Cookie>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, value, domain, path, expires, secure, httponly 
             FROM cookies 
             WHERE domain = ? OR domain LIKE ?"
        )?;
        
        let wildcard = format!(".{}", domain);
        let cookies = stmt.query_map([&domain, &wildcard], |row| {
            Ok(Cookie {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                domain: row.get(3)?,
                path: row.get(4)?,
                expires: row.get(5)?,
                secure: row.get(6)?,
                httponly: row.get(7)?,
            })
        })?;
        
        cookies.collect()
    }
    
    pub fn clear_expired_cookies(&self) -> Result<usize> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let changes = self.conn.execute(
            "DELETE FROM cookies WHERE expires IS NOT NULL AND expires < ?",
            [now],
        )?;
        
        Ok(changes)
    }
    
    pub fn clear_all_cookies(&self) -> Result<()> {
        self.conn.execute("DELETE FROM cookies", [])?;
        Ok(())
    }
    
    // Session operations
    pub fn save_session(&self, window_id: &str, tabs_json: &str, active_tab_index: i32) -> Result<i64> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Remove old session for this window
        self.conn.execute(
            "DELETE FROM sessions WHERE window_id = ?",
            [window_id],
        )?;
        
        self.conn.execute(
            "INSERT INTO sessions (window_id, tabs_json, active_tab_index, timestamp) 
             VALUES (?, ?, ?, ?)",
            [window_id, tabs_json, active_tab_index, timestamp],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn load_session(&self, window_id: &str) -> Result<Option<SessionData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, window_id, tabs_json, active_tab_index, timestamp 
             FROM sessions 
             WHERE window_id = ?"
        )?;
        
        let sessions = stmt.query_map([window_id], |row| {
            Ok(SessionData {
                id: row.get(0)?,
                window_id: row.get(1)?,
                tabs_json: row.get(2)?,
                active_tab_index: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        
        sessions.next().transpose()
    }
    
    pub fn get_all_sessions(&self) -> Result<Vec<SessionData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, window_id, tabs_json, active_tab_index, timestamp 
             FROM sessions 
             ORDER BY timestamp DESC"
        )?;
        
        let sessions = stmt.query_map([], |row| {
            Ok(SessionData {
                id: row.get(0)?,
                window_id: row.get(1)?,
                tabs_json: row.get(2)?,
                active_tab_index: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        
        sessions.collect()
    }
    
    pub fn remove_session(&self, window_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM sessions WHERE window_id = ?",
            [window_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn test_database_creation() {
        let temp_path = PathBuf::from("/tmp/test_glycerin.db");
        let _ = fs::remove_file(&temp_path);
        
        let db = DatabaseManager::new(temp_path.clone()).unwrap();
        assert!(temp_path.exists());
        
        fs::remove_file(temp_path).unwrap();
    }
    
    #[test]
    fn test_history_operations() {
        let temp_path = PathBuf::from("/tmp/test_history.db");
        let _ = fs::remove_file(&temp_path);
        
        let db = DatabaseManager::new(temp_path.clone()).unwrap();
        
        db.add_history_entry("https://example.com", "Example").unwrap();
        db.add_history_entry("https://rust-lang.org", "Rust").unwrap();
        
        let history = db.get_history(10).unwrap();
        assert_eq!(history.len(), 2);
        
        let search_results = db.search_history("rust", 10).unwrap();
        assert_eq!(search_results.len(), 1);
        
        fs::remove_file(temp_path).unwrap();
    }
    
    #[test]
    fn test_bookmark_operations() {
        let temp_path = PathBuf::from("/tmp/test_bookmarks.db");
        let _ = fs::remove_file(&temp_path);
        
        let db = DatabaseManager::new(temp_path.clone()).unwrap();
        
        let id = db.add_bookmark("https://example.com", "Example", "Favorites").unwrap();
        assert!(id > 0);
        
        let bookmarks = db.get_bookmarks(Some("Favorites")).unwrap();
        assert_eq!(bookmarks.len(), 1);
        
        db.remove_bookmark(id).unwrap();
        let bookmarks = db.get_bookmarks(Some("Favorites")).unwrap();
        assert_eq!(bookmarks.len(), 0);
        
        fs::remove_file(temp_path).unwrap();
    }
}
