//! LspRegistry — manages LSP server instances per language.

use std::collections::HashMap;
use std::path::Path;

use super::client::LspClient;
use super::protocol;

/// Configuration for a language server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

/// Registry of LSP servers — one per language.
pub struct LspRegistry {
    configs: HashMap<String, ServerConfig>,
    active: HashMap<String, LspClient>,
    disabled: Vec<String>,
    pub last_error: Option<String>,
}

impl LspRegistry {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        for (lang, cmd, args) in Self::defaults() {
            configs.insert(
                lang.to_string(),
                ServerConfig {
                    command: cmd.to_string(),
                    args: args.iter().map(|s| s.to_string()).collect(),
                },
            );
        }
        Self {
            configs,
            active: HashMap::new(),
            disabled: Vec::new(),
            last_error: None,
        }
    }

    fn defaults() -> Vec<(&'static str, &'static str, &'static [&'static str])> {
        vec![
            ("rust", "rust-analyzer", &[]),
            ("go", "gopls", &[]),
            ("typescript", "typescript-language-server", &["--stdio"]),
            ("javascript", "typescript-language-server", &["--stdio"]),
            ("c", "clangd", &[]),
            ("cpp", "clangd", &[]),
            ("java", "jdtls", &[]),
            ("python", "pyright-langserver", &["--stdio"]),
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
        self.disabled.push(language_id.to_string());
    }

    /// Get or start the LSP client for a language. Returns None if disabled or spawn fails.
    pub fn get_or_start(&mut self, language_id: &str, root_dir: &Path) -> Option<&mut LspClient> {
        if self.disabled.contains(&language_id.to_string()) {
            log::debug!("LSP disabled for {language_id}");
            return None;
        }
        if self.active.contains_key(language_id) {
            return self.active.get_mut(language_id);
        }
        let config = match self.configs.get(language_id) {
            Some(c) => c.clone(),
            None => {
                let err = format!("No LSP server configured for {language_id}");
                log::warn!("{}", err);
                self.last_error = Some(err);
                return None;
            }
        };
        let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
        let mut client = match LspClient::spawn(&config.command, &args) {
            Some(c) => c,
            None => {
                let err = format!("LSP: {} not found (is it installed?)", config.command);
                log::error!("{}", err);
                self.last_error = Some(err);
                self.disabled.push(language_id.to_string());
                return None;
            }
        };
        log::info!("LSP started: {} for {language_id}", config.command);
        let root_uri = protocol::path_to_uri(root_dir);
        protocol::initialize(&mut client, &root_uri);
        self.active.insert(language_id.to_string(), client);
        self.active.get_mut(language_id)
    }

    /// Poll all active clients for messages.
    pub fn poll_all(&self) -> Vec<(String, super::messages::LspMessage)> {
        let mut all = Vec::new();
        for (lang, client) in &self.active {
            for msg in client.poll() {
                all.push((lang.clone(), msg));
            }
        }
        all
    }

    /// Shutdown all active servers.
    pub fn shutdown_all(&mut self) {
        for client in self.active.values_mut() {
            client.send_request("shutdown", serde_json::json!(null));
            client.send_notification("exit", serde_json::json!(null));
        }
        self.active.clear();
    }

    /// List active server languages.
    pub fn active_languages(&self) -> Vec<&str> {
        self.active.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a config exists for a language.
    pub fn has_config(&self, language_id: &str) -> bool {
        self.configs.contains_key(language_id)
    }
}

impl Default for LspRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
        let result = reg.get_or_start("rust", Path::new("/tmp"));
        assert!(result.is_none());
    }

    #[test]
    fn get_or_start_nonexistent_language() {
        let mut reg = LspRegistry::new();
        let result = reg.get_or_start("brainfuck", Path::new("/tmp"));
        assert!(result.is_none());
    }

    #[test]
    fn get_or_start_missing_binary() {
        let mut reg = LspRegistry::new();
        // rust-analyzer likely not in PATH in test env
        let result = reg.get_or_start("rust", Path::new("/tmp"));
        // Either None (not installed) or Some (installed) — both are valid
        // The key is it doesn't panic
        let _ = result;
    }

    #[test]
    fn shutdown_all_clears_active() {
        let mut reg = LspRegistry::new();
        reg.shutdown_all();
        assert!(reg.active_languages().is_empty());
    }
}
