//! WebAssembly extension system facade.
use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub background_script: Option<String>,
    pub content_scripts: Vec<ContentScript>,
}
#[derive(Debug, Clone)]
pub struct ContentScript {
    pub matches: Vec<String>,
    pub js_file: String,
    pub css_file: Option<String>,
    pub run_at: RunAt,
}
#[derive(Debug, Clone, PartialEq)]
pub enum RunAt {
    DocumentStart,
    DocumentEnd,
    DocumentIdle,
}
pub struct ExtensionEngine {
    installed_extensions: HashMap<String, ExtensionManifest>,
    active: HashMap<String, bool>,
}
impl ExtensionEngine {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self {
            installed_extensions: HashMap::new(),
            active: HashMap::new(),
        })
    }
    pub fn install(
        &mut self,
        manifest: ExtensionManifest,
        _wasm_blob: Option<&[u8]>,
    ) -> Result<(), anyhow::Error> {
        self.installed_extensions
            .insert(manifest.id.clone(), manifest);
        Ok(())
    }
    pub fn activate(&mut self, ext_id: &str, tab_id: u64, _url: &str) -> Result<(), anyhow::Error> {
        if !self.installed_extensions.contains_key(ext_id) {
            anyhow::bail!("Extension not found");
        }
        self.active.insert(format!("{}-{}", ext_id, tab_id), true);
        Ok(())
    }
    pub fn get_injectable_scripts(&self, url: &str) -> Vec<String> {
        self.installed_extensions
            .values()
            .flat_map(|m| &m.content_scripts)
            .filter(|s| s.matches.iter().any(|p| url_matches_pattern(url, p)))
            .map(|s| s.js_file.clone())
            .collect()
    }
    pub fn disable(&mut self, ext_id: &str) {
        self.active.retain(|k, _| !k.starts_with(ext_id));
    }
    pub fn is_installed(&self, ext_id: &str) -> bool {
        self.installed_extensions.contains_key(ext_id)
    }
}
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    pattern == "<all_urls>" || pattern == "*://*/*" || url.contains(pattern.trim_matches('*'))
}
