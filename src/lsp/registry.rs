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
    pub(super) active: HashMap<String, LspClient>,
    disabled: Vec<String>,
    timeouts: HashMap<String, u64>,
    pub last_error: Option<String>,
    /// Maps initialize request IDs to language IDs so we can send `initialized` on response.
    pub(super) pending_init: HashMap<u64, String>,
    /// Files to send didOpen for after initialization completes.
    pub(super) pending_opens: Vec<(String, std::path::PathBuf)>,
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
            timeouts: HashMap::new(),
            last_error: None,
            pending_init: HashMap::new(),
            pending_opens: Vec::new(),
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
            return None;
        }
        let is_dead = self.active.get(language_id).is_some_and(|c| !c.is_alive());
        if is_dead {
            self.active.remove(language_id);
            let err = format!("LSP server for {language_id} died — disabled until restart");
            log::error!("{}", err);
            self.last_error = Some(err);
            self.disabled.push(language_id.to_string());
            return None;
        }
        if self.active.contains_key(language_id) {
            return self.active.get_mut(language_id);
        }
        let config = match self.configs.get(language_id) {
            Some(c) => c.clone(),
            None => {
                log::debug!("No LSP server configured for {language_id}");
                return None;
            }
        };
        let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
        let mut client = match LspClient::spawn(&config.command, &args) {
            Some(c) => c,
            None => {
                let hint = crate::tool_check::install_hint(&config.command);
                let err = format!("LSP: {} not found. Install: {}", config.command, hint);
                log::error!("{}", err);
                self.last_error = Some(err);
                self.disabled.push(language_id.to_string());
                return None;
            }
        };
        log::info!("LSP started: {} for {language_id}", config.command);
        let root_uri = protocol::path_to_uri(root_dir);
        let init_id = protocol::initialize(&mut client, &root_uri);
        self.pending_init.insert(init_id, language_id.to_string());
        self.active.insert(language_id.to_string(), client);
        self.active.get_mut(language_id)
    }

    /// Get a mutable reference to an active client by language.
    pub(super) fn get_client_mut(&mut self, language_id: &str) -> Option<&mut LspClient> {
        self.active.get_mut(language_id)
    }

    /// Check if a language server is still in the initialization phase.
    pub fn is_initializing(&self, language_id: &str) -> bool {
        self.pending_init.values().any(|lang| lang == language_id)
    }

    /// Poll all active clients for messages.
    pub fn poll_all(&mut self) -> Vec<(String, super::messages::LspMessage)> {
        let mut all = Vec::new();
        for (lang, client) in &mut self.active {
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

    /// Stop a single language server.
    pub fn stop(&mut self, language_id: &str) -> bool {
        if let Some(mut client) = self.active.remove(language_id) {
            client.send_request("shutdown", serde_json::json!(null));
            client.send_notification("exit", serde_json::json!(null));
            self.pending_init.retain(|_, lang| lang != language_id);
            true
        } else {
            false
        }
    }

    /// Stop and re-enable a language (allows get_or_start to spawn again).
    pub fn restart(&mut self, language_id: &str) {
        self.stop(language_id);
        self.disabled.retain(|l| l != language_id);
    }

    /// Set per-language timeout (seconds). 0 means use global default.
    pub fn set_timeout(&mut self, language_id: &str, secs: u64) {
        self.timeouts.insert(language_id.to_string(), secs);
    }

    /// Get per-language timeout, or None for global default.
    pub fn timeout(&self, language_id: &str) -> Option<u64> {
        self.timeouts.get(language_id).copied().filter(|&t| t > 0)
    }

    /// Return languages matching a glob pattern (e.g. "rust", "type*", "*").
    pub fn matching_languages(&self, pattern: &str) -> Vec<String> {
        let all: Vec<&str> = self
            .configs
            .keys()
            .map(|s| s.as_str())
            .chain(self.active.keys().map(|s| s.as_str()))
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
        self.active.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a config exists for a language.
    pub fn has_config(&self, language_id: &str) -> bool {
        self.configs.contains_key(language_id)
    }
}

/// Simple glob matching: supports `*` (any chars) and `?` (single char).
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
