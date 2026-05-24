//! LspRegistry — configuration, query, and lifecycle management methods.

use super::registry::{LspRegistry, ServerConfig, ServerState};

impl LspRegistry {
    pub(super) fn defaults() -> Vec<(&'static str, &'static str, &'static [&'static str])> {
        vec![
            ("rust", "rust-analyzer", &[]),
            ("go", "gopls", &[]),
            ("typescript", "typescript-language-server", &["--stdio"]),
            ("javascript", "typescript-language-server", &["--stdio"]),
            ("c", "clangd", &[]),
            ("cpp", "clangd", &[]),
            ("java", "jdtls", &[]),
            ("python", "pyright-langserver", &["--stdio"]),
            ("tcl", "rusticle-lsp", &["--prelude", ".kairn/prelude.tcl"]),
        ]
    }

    /// Set or override a server config for a language.
    pub fn set_config(&mut self, language_id: &str, command: &str, args: &[String]) {
        self.configs.insert(
            language_id.to_string(),
            ServerConfig {
                command: command.to_string(),
                args: args.to_vec(),
            },
        );
    }

    /// Disable LSP for a language.
    pub fn disable(&mut self, language_id: &str) {
        self.servers.insert(language_id.to_string(), ServerState::Disabled);
    }

    /// Set per-language timeout (seconds). 0 means use global default.
    pub fn set_timeout(&mut self, language_id: &str, secs: u64) {
        self.timeouts.insert(language_id.to_string(), secs);
    }

    /// Get per-language timeout, or None for global default.
    pub fn timeout(&self, language_id: &str) -> Option<u64> {
        self.timeouts.get(language_id).copied().filter(|&t| t > 0)
    }

    /// Return languages matching a glob pattern.
    pub fn matching_languages(&self, pattern: &str) -> Vec<String> {
        let all: Vec<&str> = self
            .configs
            .keys()
            .map(|s| s.as_str())
            .chain(self.servers.keys().map(|s| s.as_str()))
            .collect();
        let mut matched: Vec<String> = all
            .into_iter()
            .filter(|lang| glob_match(pattern, lang))
            .map(|s| s.to_string())
            .collect();
        matched.sort();
        matched.dedup();
        matched
    }

    /// List active server languages.
    pub fn active_languages(&self) -> Vec<&str> {
        self.servers
            .iter()
            .filter(|(_, s)| !matches!(s, ServerState::Disabled))
            .map(|(l, _)| l.as_str())
            .collect()
    }

    /// Check if a config exists for a language.
    pub fn has_config(&self, language_id: &str) -> bool {
        self.configs.contains_key(language_id)
    }

    /// Shutdown all active servers.
    pub fn shutdown_all(&mut self) {
        for state in self.servers.values_mut() {
            if let ServerState::Starting { client, .. }
            | ServerState::WarmingUp { client }
            | ServerState::Ready { client } = state
            {
                client.send_request("shutdown", serde_json::json!(null));
                client.send_notification("exit", serde_json::json!(null));
            }
        }
        self.servers.clear();
    }

    /// Stop a single language server.
    pub fn stop(&mut self, language_id: &str) -> bool {
        if let Some(state) = self.servers.get_mut(language_id) {
            if let ServerState::Starting { client, .. }
            | ServerState::WarmingUp { client }
            | ServerState::Ready { client } = state
            {
                client.send_request("shutdown", serde_json::json!(null));
                client.send_notification("exit", serde_json::json!(null));
            }
            self.servers.remove(language_id);
            true
        } else {
            false
        }
    }

    /// Stop and re-enable a language (allows ensure_started to spawn again).
    pub fn restart(&mut self, language_id: &str) {
        self.stop(language_id);
    }
}

/// Simple glob matching: supports `*` (any chars).
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }
    pattern == text
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn new_has_defaults() {
        let reg = LspRegistry::new();
        assert!(reg.has_config("rust"));
        assert!(reg.has_config("go"));
        assert!(reg.has_config("typescript"));
        assert!(!reg.has_config("haskell"));
    }

    #[test]
    fn set_config_overrides() {
        let mut reg = LspRegistry::new();
        reg.set_config("rust", "my-analyzer", &["--flag".to_string()]);
        assert!(reg.has_config("rust"));
    }

    #[test]
    fn disable_prevents_start() {
        let mut reg = LspRegistry::new();
        reg.disable("rust");
        assert!(!reg.ensure_started("rust", Path::new("/tmp")));
    }

    #[test]
    fn get_or_start_nonexistent_language() {
        let mut reg = LspRegistry::new();
        assert!(!reg.ensure_started("brainfuck", Path::new("/tmp")));
    }

    #[test]
    fn shutdown_all_clears() {
        let mut reg = LspRegistry::new();
        reg.shutdown_all();
        assert!(reg.active_languages().is_empty());
    }
}
