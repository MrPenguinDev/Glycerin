//! Phase 5: Extension System & Plugin Architecture
//! 
//! Provides a WebAssembly-based extension API allowing users to install
//! plugins that modify browser behavior, inject scripts, and add features.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::*;

/// Metadata about an extension
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
    pub matches: Vec<String>, // URL patterns
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

/// Runtime instance of an extension
pub struct ExtensionInstance {
    pub manifest: ExtensionManifest,
    pub store: Store<ExtensionContext>,
    pub module: Option<Module>,
    pub is_active: bool,
}

pub struct ExtensionContext {
    pub tab_id: u64,
    pub url: String,
    pub console_logs: Vec<String>,
}

impl ExtensionContext {
    fn new(tab_id: u64, url: String) -> Self {
        Self {
            tab_id,
            url,
            console_logs: Vec::new(),
        }
    }
}

/// Manages the lifecycle and execution of extensions
pub struct ExtensionEngine {
    engine: wasmtime::Engine,
    linker: Linker<ExtensionContext>,
    installed_extensions: HashMap<String, ExtensionManifest>,
    active_instances: HashMap<String, ExtensionInstance>,
}

impl ExtensionEngine {
    pub fn new() -> Result<Self, anyhow::Error> {
        let engine = wasmtime::Engine::default();
        let mut linker = Linker::<ExtensionContext>::new(&engine);

        // Define host functions available to extensions
        linker.func_wrap("glycerin", "tab_get_url", |caller: Caller<'_, ExtensionContext>| {
            let ctx = caller.data();
            ctx.url.clone()
        })?;

        linker.func_wrap("glycerin", "console_log", |caller: Caller<'_, ExtensionContext>, msg: String| {
            let ctx = caller.data_mut();
            ctx.console_logs.push(msg);
        })?;

        linker.func_wrap("glycerin", "storage_get", |_caller: Caller<'_, ExtensionContext>, key: String| {
            // Simplified: return empty string for now
            String::from("")
        })?;

        Ok(Self {
            engine,
            linker,
            installed_extensions: HashMap::new(),
            active_instances: HashMap::new(),
        })
    }

    /// Install an extension from a manifest
    pub fn install(&mut self, manifest: ExtensionManifest, wasm_blob: Option<&[u8]>) -> Result<(), anyhow::Error> {
        if let Some(blob) = wasm_blob {
            let module = Module::from_binary(&self.engine, blob)?;
            // Validate module exports/imports against permissions here
            let _ = module; 
        }

        self.installed_extensions.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    /// Activate an extension for a specific tab
    pub fn activate(&mut self, ext_id: &str, tab_id: u64, url: &str) -> Result<(), anyhow::Error> {
        let manifest = self.installed_extensions.get(ext_id)
            .ok_or_else(|| anyhow::anyhow!("Extension not found"))?
            .clone();

        let mut store = Store::new(&self.engine, ExtensionContext::new(tab_id, url.to_string()));
        
        // In a real implementation, we would instantiate the WASM module here
        // let instance = self.linker.instantiate(&mut store, &module)?;

        let instance = ExtensionInstance {
            manifest,
            store,
            module: None,
            is_active: true,
        };

        self.active_instances.insert(format!("{}-{}", ext_id, tab_id), instance);
        Ok(())
    }

    /// Inject content scripts into a page based on URL matching
    pub fn get_injectable_scripts(&self, url: &str) -> Vec<String> {
        let mut scripts = Vec::new();
        
        for manifest in self.installed_extensions.values() {
            for script in &manifest.content_scripts {
                if script.matches.iter().any(|pattern| url_matches_pattern(url, pattern)) {
                    scripts.push(script.js_file.clone());
                }
            }
        }
        
        scripts
    }

    /// Disable an extension
    pub fn disable(&mut self, ext_id: &str) {
        self.active_instances.retain(|k, v| {
            if k.starts_with(ext_id) {
                v.is_active = false;
                false // Remove from active
            } else {
                true
            }
        });
    }
}

/// Simple glob-style pattern matching for content scripts
fn url_matches_pattern(url: &str, pattern: &str) -> bool {
    if pattern == "*://*/*" { return true; }
    
    let pattern = pattern.replace("*", ".*");
    if let Ok(re) = regex::Regex::new(&format!("^{}$", pattern)) {
        return re.is_match(url);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_engine_creation() {
        let engine = ExtensionEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_content_script_matching() {
        assert!(url_matches_pattern("https://example.com/page", "*://*/*"));
        assert!(url_matches_pattern("https://google.com", "*://google.com/*"));
        assert!(!url_matches_pattern("https://other.com", "*://google.com/*"));
    }

    #[test]
    fn test_install_extension() {
        let mut engine = ExtensionEngine::new().unwrap();
        let manifest = ExtensionManifest {
            id: "test-ext".to_string(),
            name: "Test Extension".to_string(),
            version: "1.0".to_string(),
            description: "A test".to_string(),
            permissions: vec!["tabs".to_string()],
            background_script: None,
            content_scripts: vec![],
        };
        
        assert!(engine.install(manifest, None).is_ok());
        assert!(engine.installed_extensions.contains_key("test-ext"));
    }
}
