//! Phase 4: Security & Sandboxing
//! 
//! Implements process isolation strategies, Content Security Policy (CSP),
//! and safe browsing heuristics to protect the user from malicious content.

use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use regex::Regex;

/// Represents the security context of a rendered page
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub origin: String,
    pub csp_policy: ContentSecurityPolicy,
    pub is_secure: bool, // HTTPS
    pub sandbox_flags: SandboxFlags,
}

/// Content Security Policy directives
#[derive(Debug, Clone, Default)]
pub struct ContentSecurityPolicy {
    pub default_src: Option<Vec<String>>,
    pub script_src: Option<Vec<String>>,
    pub style_src: Option<Vec<String>>,
    pub img_src: Option<Vec<String>>,
    pub connect_src: Option<Vec<String>>,
    pub frame_ancestors: Option<Vec<String>>,
    pub report_uri: Option<String>,
}

impl ContentSecurityPolicy {
    /// Parse a CSP header string into a structured policy
    pub fn parse(header: &str) -> Self {
        let mut policy = ContentSecurityPolicy::default();
        
        for directive in header.split(';') {
            let parts: Vec<&str> = directive.trim().split_whitespace().collect();
            if parts.is_empty() { continue; }
            
            let name = parts[0].to_lowercase();
            let values: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            
            match name.as_str() {
                "default-src" => policy.default_src = Some(values),
                "script-src" => policy.script_src = Some(values),
                "style-src" => policy.style_src = Some(values),
                "img-src" => policy.img_src = Some(values),
                "connect-src" => policy.connect_src = Some(values),
                "frame-ancestors" => policy.frame_ancestors = Some(values),
                "report-uri" => if !values.is_empty() { policy.report_uri = Some(values[0].clone()); },
                _ => {}
            }
        }
        policy
    }

    /// Check if a resource load is allowed based on CSP
    pub fn allows(&self, resource_type: &str, url: &str) -> bool {
        let sources = match resource_type {
            "script" => self.script_src.as_ref().or(self.default_src.as_ref()),
            "style" => self.style_src.as_ref().or(self.default_src.as_ref()),
            "image" => self.img_src.as_ref().or(self.default_src.as_ref()),
            "xhr" => self.connect_src.as_ref().or(self.default_src.as_ref()),
            _ => self.default_src.as_ref(),
        };

        match sources {
            None => true, // No policy defined, allow all
            Some(list) => {
                if list.contains(&"'self'".to_string()) {
                    // Simplified check: assumes same origin if URL starts with origin
                    // In real impl, compare origins strictly
                    return true; 
                }
                if list.contains(&"'unsafe-inline'".to_string()) && resource_type == "script" {
                    return true;
                }
                // Check wildcards or specific domains
                list.iter().any(|pattern| url.contains(pattern))
            }
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct SandboxFlags: u32 {
        const NONE = 0;
        const ALLOW_SCRIPTS = 1 << 0;
        const ALLOW_FORMS = 1 << 1;
        const ALLOW_POPUPS = 1 << 2;
        const ALLOW_TOP_NAVIGATION = 1 << 3;
        const ALLOW_SAME_ORIGIN = 1 << 4;
        const STRICT_ISOLATION = 1 << 5;
    }
}

/// Manages safe browsing lists and heuristic checks
pub struct SafeBrowsingManager {
    malicious_domains: RwLock<HashSet<String>>,
    phishing_patterns: Vec<Regex>,
}

impl SafeBrowsingManager {
    pub fn new() -> Self {
        let mut patterns = Vec::new();
        // Basic heuristic patterns for demonstration
        if let Ok(re) = Regex::new(r"paypa[l1]\.com") { patterns.push(re); }
        if let Ok(re) = Regex::new(r"amaz0n\.com") { patterns.push(re); }
        
        Self {
            malicious_domains: RwLock::new(HashSet::new()),
            phishing_patterns: patterns,
        }
    }

    /// Add a domain to the local blocklist
    pub fn add_malicious_domain(&self, domain: &str) {
        if let Ok(mut set) = self.malicious_domains.write() {
            set.insert(domain.to_lowercase());
        }
    }

    /// Check if a URL is safe to load
    pub fn is_safe(&self, url: &str) -> bool {
        let lower_url = url.to_lowercase();
        
        // Check blocklist
        if let Ok(set) = self.malicious_domains.read() {
            for domain in set.iter() {
                if lower_url.contains(domain) {
                    return false;
                }
            }
        }

        // Check heuristics
        for pattern in &self.phishing_patterns {
            if pattern.is_match(&lower_url) {
                return false;
            }
        }

        true
    }
}

/// Simulates process isolation boundaries
pub struct ProcessIsolator {
    site_per_process: bool,
}

impl ProcessIsolator {
    pub fn new(site_per_process: bool) -> Self {
        Self { site_per_process }
    }

    /// Determine if a new process should be spawned for this navigation
    pub fn requires_new_process(&self, current_origin: &str, new_origin: &str) -> bool {
        if !self.site_per_process {
            return false;
        }
        fn origin(input: &str) -> &str {
            let after_scheme = input.split_once("://").map(|(_, rest)| rest).unwrap_or(input);
            after_scheme.split('/').next().unwrap_or(after_scheme)
        }
        origin(current_origin) != origin(new_origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csp_parsing() {
        let header = "default-src 'self'; script-src 'unsafe-inline' https://cdn.example.com";
        let policy = ContentSecurityPolicy::parse(header);
        
        assert!(policy.default_src.is_some());
        assert!(policy.script_src.is_some());
        assert_eq!(policy.report_uri, None);
    }

    #[test]
    fn test_safe_browsing() {
        let manager = SafeBrowsingManager::new();
        manager.add_malicious_domain("malware-site.com");
        
        assert!(manager.is_safe("https://google.com"));
        assert!(!manager.is_safe("https://malware-site.com/payload"));
        assert!(!manager.is_safe("https://paypa1.com/phish"));
    }

    #[test]
    fn test_process_isolation() {
        let isolator = ProcessIsolator::new(true);
        assert!(isolator.requires_new_process("https://a.com", "https://b.com"));
        assert!(!isolator.requires_new_process("https://a.com", "https://a.com/page"));
    }
}
