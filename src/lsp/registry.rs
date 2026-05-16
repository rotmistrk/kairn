//! LspRegistry — manages LSP server lifecycle per language via state machine.
//!
//! States: Starting → WarmingUp → Ready → Disabled
//! No out-of-order messages possible: requests only sent in Ready state.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::client::LspClient;
use super::protocol;

/// Configuration for a language server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

/// Lifecycle state of a single LSP server.
pub(super) enum ServerState {
    /// initialize request sent, waiting for response.
    Starting { client: LspClient, init_id: u64 },
    /// initialized notification sent, waiting one tick for server to process it.
    WarmingUp { client: LspClient },
    /// Server is ready to accept requests.
    Ready { client: LspClient },
    /// Server died or was disabled.
    Disabled,
}

/// Registry of LSP servers — one per language.
pub struct LspRegistry {
    pub(super) configs: HashMap<String, ServerConfig>,
    pub(super) servers: HashMap<String, ServerState>,
    pub(super) timeouts: HashMap<String, u64>,
    pub last_error: Option<String>,
    /// Files to send didOpen for after initialization completes.
    pub(super) pending_opens: Vec<(String, PathBuf)>,
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
            servers: HashMap::new(),
            timeouts: HashMap::new(),
            last_error: None,
            pending_opens: Vec::new(),
        }
    }

    /// Ensure a server is started for a language. Returns true if ready for requests.
    pub fn ensure_started(&mut self, language_id: &str, root_dir: &Path) -> bool {
        if let Some(state) = self.servers.get(language_id) {
            return matches!(state, ServerState::Ready { .. });
        }
        let config = match self.configs.get(language_id) {
            Some(c) => c.clone(),
            None => return false,
        };
        let args: Vec<&str> = config.args.iter().map(|s| s.as_str()).collect();
        let mut client = match LspClient::spawn(&config.command, &args) {
            Some(c) => c,
            None => {
                let hint = crate::tool_check::install_hint(&config.command);
                let err = format!("LSP: {} not found. Install: {}", config.command, hint);
                log::error!("{err}");
                self.last_error = Some(err);
                self.servers.insert(language_id.to_string(), ServerState::Disabled);
                return false;
            }
        };
        log::info!("LSP started: {} for {language_id}", config.command);
        let root_uri = protocol::path_to_uri(root_dir);
        let init_id = protocol::initialize(&mut client, &root_uri);
        self.servers
            .insert(language_id.to_string(), ServerState::Starting { client, init_id });
        false
    }

    /// Get a mutable reference to the client, only if Ready.
    pub(super) fn get_client_mut(&mut self, language_id: &str) -> Option<&mut LspClient> {
        match self.servers.get_mut(language_id) {
            Some(ServerState::Ready { client }) => Some(client),
            _ => None,
        }
    }

    /// Check if a language server is NOT ready for requests.
    pub fn is_initializing(&self, language_id: &str) -> bool {
        matches!(
            self.servers.get(language_id),
            Some(ServerState::Starting { .. }) | Some(ServerState::WarmingUp { .. }) | None
        )
    }

    /// Check if a response ID matches a Starting server. Returns the language.
    pub(super) fn is_init_response(&self, id: u64) -> Option<String> {
        self.servers.iter().find_map(|(lang, state)| {
            if let ServerState::Starting { init_id, .. } = state {
                if *init_id == id {
                    return Some(lang.clone());
                }
            }
            None
        })
    }

    /// Transition Starting → WarmingUp: send initialized notification.
    pub(super) fn complete_init(&mut self, language_id: &str) {
        let Some(state) = self.servers.remove(language_id) else {
            return;
        };
        if let ServerState::Starting { mut client, .. } = state {
            protocol::initialized(&mut client);
            log::info!("Sent initialized notification for {language_id}");
            self.servers
                .insert(language_id.to_string(), ServerState::WarmingUp { client });
        }
    }

    /// Transition Starting → Disabled on init failure.
    pub(super) fn fail_init(&mut self, language_id: &str) {
        self.servers.insert(language_id.to_string(), ServerState::Disabled);
    }

    /// Transition WarmingUp → Ready (called at start of next tick).
    pub(super) fn advance_warming_up(&mut self) -> Vec<String> {
        let warming: Vec<String> = self
            .servers
            .iter()
            .filter(|(_, s)| matches!(s, ServerState::WarmingUp { .. }))
            .map(|(l, _)| l.clone())
            .collect();
        for lang in &warming {
            if let Some(ServerState::WarmingUp { client }) = self.servers.remove(lang) {
                self.servers.insert(lang.clone(), ServerState::Ready { client });
            }
        }
        warming
    }

    /// Check for dead servers and transition them to Disabled.
    pub(super) fn detect_dead(&mut self) -> Vec<String> {
        let dead: Vec<String> = self
            .servers
            .iter()
            .filter(|(_, s)| match s {
                ServerState::Starting { client, .. }
                | ServerState::WarmingUp { client }
                | ServerState::Ready { client } => !client.is_alive(),
                ServerState::Disabled => false,
            })
            .map(|(l, _)| l.clone())
            .collect();
        for lang in &dead {
            self.servers.insert(lang.clone(), ServerState::Disabled);
        }
        dead
    }

    /// Poll all active clients for messages.
    pub fn poll_all(&mut self) -> Vec<(String, super::messages::LspMessage)> {
        let mut all = Vec::new();
        for (lang, state) in &mut self.servers {
            let client = match state {
                ServerState::Starting { client, .. }
                | ServerState::WarmingUp { client }
                | ServerState::Ready { client } => client,
                ServerState::Disabled => continue,
            };
            for msg in client.poll() {
                all.push((lang.clone(), msg));
            }
        }
        all
    }
}

impl Default for LspRegistry {
    fn default() -> Self {
        Self::new()
    }
}
