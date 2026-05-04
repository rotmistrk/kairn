//! LSP client for kairn — communicates with language servers over stdio.
//!
//! `LspClient` lives on the main thread. It exposes synchronous methods
//! that send requests through channels and return immediately. Per-server
//! reader/writer tasks run on the tokio runtime.

pub mod protocol;
pub mod transport;
pub mod types;

pub mod capabilities;

use std::collections::HashMap;

use tokio::sync::mpsc;

use protocol::{LspMessage, LspNotification, LspRequest, RequestId};
use types::{
    DebounceState, DocumentUri, DocumentVersion, LanguageId, LspEvent, OpenDocSet, PendingRequest,
    ServerConfig, ServerState,
};

/// Channel capacity: main thread → writer task.
const REQUEST_CHANNEL_SIZE: usize = 64;
/// Channel capacity: reader task → main thread.
const EVENT_CHANNEL_SIZE: usize = 256;
/// Channel capacity: pending method registry for reader task.
const PENDING_CHANNEL_SIZE: usize = 128;
/// Maximum crash count before disabling a server.
const MAX_CRASH_COUNT: u32 = 2;
/// Minimum uptime (seconds) before a crash resets the counter.
const CRASH_WINDOW_SECS: u64 = 30;

/// One running language server process.
struct LspServer {
    language_id: LanguageId,
    state: ServerState,
    /// Send outgoing messages to the writer task.
    request_tx: mpsc::Sender<LspMessage>,
    /// Send pending method info to the reader task.
    pending_tx: mpsc::Sender<(RequestId, String, Option<DocumentUri>)>,
    /// Next request ID (monotonically increasing).
    next_id: u64,
    /// Open document URIs managed by this server.
    open_docs: OpenDocSet,
    /// Child process handle.
    child: tokio::process::Child,
    /// Crash count for this session.
    crash_count: u32,
    /// Timestamp of last crash.
    last_crash: Option<std::time::Instant>,
    /// Server capabilities from initialize response.
    capabilities: Option<serde_json::Value>,
}

/// Main-thread LSP client facade.
pub struct LspClient {
    /// Server configs keyed by language_id.
    registry: HashMap<LanguageId, ServerConfig>,
    /// Extension → language_id lookup.
    ext_map: HashMap<String, LanguageId>,
    /// Active server processes keyed by language_id.
    servers: HashMap<LanguageId, LspServer>,
    /// Receive events from all server reader tasks.
    event_rx: mpsc::Receiver<LspEvent>,
    /// Sender cloned into each server's reader task.
    event_tx: mpsc::Sender<LspEvent>,
    /// Workspace root URI.
    workspace_root: Option<DocumentUri>,
    /// Debounce state per document.
    debounce: HashMap<DocumentUri, DebounceState>,
    /// Tokio runtime handle for spawning tasks.
    rt: tokio::runtime::Handle,
}

// ── Construction and registry ───────────────────────────────

impl LspClient {
    /// Create a new LSP client with the given workspace root.
    pub fn new(workspace_root: Option<&str>, rt: tokio::runtime::Handle) -> Self {
        let (event_tx, event_rx) = mpsc::channel(EVENT_CHANNEL_SIZE);
        let mut client = Self {
            registry: HashMap::new(),
            ext_map: HashMap::new(),
            servers: HashMap::new(),
            event_rx,
            event_tx,
            workspace_root: workspace_root.map(DocumentUri::from_path),
            debounce: HashMap::new(),
            rt,
        };
        client.register_defaults();
        client
    }

    /// Register a server config. Called at startup.
    pub fn register(&mut self, config: ServerConfig) {
        for ext in &config.extensions {
            self.ext_map.insert(ext.clone(), config.language_id.clone());
        }
        self.registry.insert(config.language_id.clone(), config);
    }

    /// Look up language_id for a file extension.
    pub fn language_for_extension(&self, ext: &str) -> Option<&LanguageId> {
        self.ext_map.get(ext)
    }

    /// Whether a server is ready for a given language.
    pub fn is_ready(&self, language_id: &LanguageId) -> bool {
        self.servers
            .get(language_id)
            .is_some_and(|s| s.state == ServerState::Ready)
    }

    /// Register default server configs per the spec.
    fn register_defaults(&mut self) {
        let defaults = [
            ServerConfig {
                language_id: LanguageId::new("rust"),
                command: "rust-analyzer".into(),
                args: vec![],
                extensions: vec![".rs".into()],
                init_options: None,
                root_markers: vec!["Cargo.toml".into()],
            },
            ServerConfig {
                language_id: LanguageId::new("go"),
                command: "gopls".into(),
                args: vec!["serve".into()],
                extensions: vec![".go".into()],
                init_options: None,
                root_markers: vec!["go.mod".into()],
            },
            ServerConfig {
                language_id: LanguageId::new("typescript"),
                command: "typescript-language-server".into(),
                args: vec!["--stdio".into()],
                extensions: vec![".ts".into(), ".tsx".into()],
                init_options: None,
                root_markers: vec!["tsconfig.json".into(), "package.json".into()],
            },
            ServerConfig {
                language_id: LanguageId::new("javascript"),
                command: "typescript-language-server".into(),
                args: vec!["--stdio".into()],
                extensions: vec![".js".into(), ".jsx".into()],
                init_options: None,
                root_markers: vec!["package.json".into()],
            },
        ];
        for config in defaults {
            self.register(config);
        }
    }
}

// ── Server lifecycle ────────────────────────────────────────

impl LspClient {
    /// Spawn a language server process and its reader/writer tasks.
    fn spawn_server(&mut self, language_id: &LanguageId) -> Result<(), String> {
        let config = self
            .registry
            .get(language_id)
            .ok_or_else(|| format!("no config for {language_id}"))?
            .clone();

        // Check if disabled.
        if let Some(existing) = self.servers.get(language_id) {
            if existing.state == ServerState::Disabled {
                return Err(format!("{language_id} is disabled"));
            }
        }

        let mut cmd = tokio::process::Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to start {}: {e}", config.command))?;

        let stdin = child.stdin.take().ok_or("no stdin")?;
        let stdout = child.stdout.take().ok_or("no stdout")?;

        let (req_tx, req_rx) = mpsc::channel(REQUEST_CHANNEL_SIZE);
        let (pending_tx, pending_rx) = mpsc::channel(PENDING_CHANNEL_SIZE);

        // Spawn writer task.
        self.rt.spawn(transport::writer_task(req_rx, stdin));

        // Spawn reader task.
        let lang_clone = language_id.clone();
        let event_tx = self.event_tx.clone();
        self.rt.spawn(transport::reader_task(
            lang_clone, stdout, pending_rx, event_tx,
        ));

        let server = LspServer {
            language_id: language_id.clone(),
            state: ServerState::Starting,
            request_tx: req_tx,
            pending_tx,
            next_id: 1,
            open_docs: OpenDocSet::default(),
            child,
            crash_count: 0,
            last_crash: None,
            capabilities: None,
        };

        self.servers.insert(language_id.clone(), server);
        Ok(())
    }

    /// Send shutdown + exit to a server and clean up.
    fn shutdown_server(&mut self, language_id: &LanguageId) {
        let Some(server) = self.servers.get_mut(language_id) else {
            return;
        };
        if server.state != ServerState::Ready {
            self.servers.remove(language_id);
            return;
        }

        server.state = ServerState::ShuttingDown;

        // Send shutdown request.
        let id = RequestId::new(server.next_id);
        server.next_id += 1;
        let shutdown_req = LspMessage::Request(LspRequest {
            id,
            method: "shutdown".into(),
            params: serde_json::Value::Null,
        });
        let tx = server.request_tx.clone();
        self.rt.spawn(async move {
            let _ = tx.send(shutdown_req).await;
        });

        // Send exit notification after a short delay.
        let tx2 = server.request_tx.clone();
        self.rt.spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let exit = LspMessage::Notification(LspNotification {
                method: "exit".into(),
                params: serde_json::Value::Null,
            });
            let _ = tx2.send(exit).await;
        });

        // Schedule cleanup.
        let lang = language_id.clone();
        let event_tx = self.event_tx.clone();
        let mut child = self.servers.remove(&lang);
        self.rt.spawn(async move {
            if let Some(ref mut srv) = child {
                // Wait up to 5 seconds for process to exit.
                let timeout =
                    tokio::time::timeout(std::time::Duration::from_secs(5), srv.child.wait());
                if timeout.await.is_err() {
                    let _ = srv.child.kill().await;
                }
            }
            // Notify main thread that server is gone (via crash event
            // if it didn't exit cleanly — reader task handles that).
            drop(event_tx);
        });
    }

    /// Handle a server crash: mark state, attempt restart if allowed.
    pub fn handle_crash(&mut self, language_id: &LanguageId) {
        let should_disable = if let Some(server) = self.servers.get_mut(language_id) {
            server.state = ServerState::Crashed;
            server.crash_count += 1;

            let recent = server
                .last_crash
                .is_some_and(|t| t.elapsed().as_secs() < CRASH_WINDOW_SECS);
            server.last_crash = Some(std::time::Instant::now());

            recent && server.crash_count >= MAX_CRASH_COUNT
        } else {
            false
        };

        if should_disable {
            if let Some(server) = self.servers.get_mut(language_id) {
                server.state = ServerState::Disabled;
                tracing::warn!("LSP server {language_id} disabled after repeated crashes");
            }
        }
    }
}

// ── Document lifecycle ──────────────────────────────────────

impl LspClient {
    /// Notify that a file was opened. Starts server if needed.
    pub fn file_opened(
        &mut self,
        uri: &DocumentUri,
        language_id: &LanguageId,
        content: &str,
    ) -> Result<(), String> {
        // Ensure server is running.
        if !self.servers.contains_key(language_id) {
            self.spawn_server(language_id)?;
            // Queue the initialize request.
            self.send_initialize(language_id)?;
        }

        let server = self
            .servers
            .get_mut(language_id)
            .ok_or("server not found")?;

        server.open_docs.insert(uri.clone());

        // Send didOpen (will be queued; server may still be starting).
        let params = serde_json::json!({
            "textDocument": {
                "uri": uri.as_str(),
                "languageId": language_id.as_str(),
                "version": 0,
                "text": content,
            }
        });
        let notif = LspMessage::Notification(LspNotification {
            method: "textDocument/didOpen".into(),
            params,
        });
        self.send_to_server(language_id, notif);

        // Initialize debounce state.
        self.debounce.insert(
            uri.clone(),
            DebounceState {
                pending_changes: Vec::new(),
                last_edit: std::time::Instant::now(),
                version: DocumentVersion::new(0),
            },
        );

        Ok(())
    }

    /// Send incremental changes for a document.
    pub fn file_changed(
        &mut self,
        uri: &DocumentUri,
        version: DocumentVersion,
        changes: &[crate::buffer::TextChange],
    ) -> Result<(), String> {
        let lang = self.language_for_uri(uri).cloned();
        let Some(language_id) = lang else {
            return Ok(()); // No LSP for this file.
        };

        let content_changes: Vec<serde_json::Value> = changes
            .iter()
            .map(|c| {
                serde_json::json!({
                    "range": {
                        "start": {
                            "line": c.start_line,
                            "character": c.start_col,
                        },
                        "end": {
                            "line": c.end_line,
                            "character": c.end_col,
                        },
                    },
                    "text": c.new_text,
                })
            })
            .collect();

        let params = serde_json::json!({
            "textDocument": {
                "uri": uri.as_str(),
                "version": version.value(),
            },
            "contentChanges": content_changes,
        });

        let notif = LspMessage::Notification(LspNotification {
            method: "textDocument/didChange".into(),
            params,
        });
        self.send_to_server(&language_id, notif);
        Ok(())
    }

    /// Notify that a file was saved.
    pub fn file_saved(&mut self, uri: &DocumentUri, text: Option<&str>) -> Result<(), String> {
        let lang = self.language_for_uri(uri).cloned();
        let Some(language_id) = lang else {
            return Ok(());
        };

        let mut params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()}
        });
        if let Some(t) = text {
            params["text"] = serde_json::Value::String(t.to_string());
        }

        let notif = LspMessage::Notification(LspNotification {
            method: "textDocument/didSave".into(),
            params,
        });
        self.send_to_server(&language_id, notif);
        Ok(())
    }

    /// Notify that a file was closed. May shut down server.
    pub fn file_closed(&mut self, uri: &DocumentUri) -> Result<(), String> {
        let lang = self.language_for_uri(uri).cloned();
        let Some(language_id) = lang else {
            return Ok(());
        };

        // Send didClose.
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()}
        });
        let notif = LspMessage::Notification(LspNotification {
            method: "textDocument/didClose".into(),
            params,
        });
        self.send_to_server(&language_id, notif);

        // Remove from open docs.
        if let Some(server) = self.servers.get_mut(&language_id) {
            server.open_docs.remove(uri);
            if server.open_docs.is_empty() {
                self.shutdown_server(&language_id);
            }
        }

        self.debounce.remove(uri);
        Ok(())
    }

    /// Find the language_id for a document URI by extension.
    fn language_for_uri(&self, uri: &DocumentUri) -> Option<&LanguageId> {
        let path = uri.as_str();
        let ext_start = path.rfind('.')?;
        let ext = &path[ext_start..];
        self.ext_map.get(ext)
    }
}

// ── Request dispatch ────────────────────────────────────────

impl LspClient {
    /// Request completions at a position.
    pub fn request_completion(
        &mut self,
        uri: &DocumentUri,
        line: u32,
        character: u32,
    ) -> Result<RequestId, String> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()},
            "position": {"line": line, "character": character},
        });
        self.send_request(uri, "textDocument/completion", params)
    }

    /// Request hover info at a position.
    pub fn request_hover(
        &mut self,
        uri: &DocumentUri,
        line: u32,
        character: u32,
    ) -> Result<RequestId, String> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()},
            "position": {"line": line, "character": character},
        });
        self.send_request(uri, "textDocument/hover", params)
    }

    /// Request go-to-definition.
    pub fn request_definition(
        &mut self,
        uri: &DocumentUri,
        line: u32,
        character: u32,
    ) -> Result<RequestId, String> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()},
            "position": {"line": line, "character": character},
        });
        self.send_request(uri, "textDocument/definition", params)
    }

    /// Request find-references.
    pub fn request_references(
        &mut self,
        uri: &DocumentUri,
        line: u32,
        character: u32,
    ) -> Result<RequestId, String> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()},
            "position": {"line": line, "character": character},
            "context": {"includeDeclaration": true},
        });
        self.send_request(uri, "textDocument/references", params)
    }

    /// Request document symbols.
    pub fn request_document_symbols(&mut self, uri: &DocumentUri) -> Result<RequestId, String> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()},
        });
        self.send_request(uri, "textDocument/documentSymbol", params)
    }

    /// Request formatting.
    pub fn request_formatting(&mut self, uri: &DocumentUri) -> Result<RequestId, String> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()},
            "options": {
                "tabSize": 4,
                "insertSpaces": true,
            },
        });
        self.send_request(uri, "textDocument/formatting", params)
    }

    /// Request rename.
    pub fn request_rename(
        &mut self,
        uri: &DocumentUri,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<RequestId, String> {
        let params = serde_json::json!({
            "textDocument": {"uri": uri.as_str()},
            "position": {"line": line, "character": character},
            "newName": new_name,
        });
        self.send_request(uri, "textDocument/rename", params)
    }
}

// ── Internal helpers ────────────────────────────────────────

impl LspClient {
    /// Send a request to the server handling the given URI.
    fn send_request(
        &mut self,
        uri: &DocumentUri,
        method: &str,
        params: serde_json::Value,
    ) -> Result<RequestId, String> {
        let lang = self
            .language_for_uri(uri)
            .cloned()
            .ok_or("no language for URI")?;

        let server = self.servers.get_mut(&lang).ok_or("server not running")?;

        if server.state != ServerState::Ready {
            return Err("server not ready".into());
        }

        let id = RequestId::new(server.next_id);
        server.next_id += 1;

        let msg = LspMessage::Request(LspRequest {
            id,
            method: method.to_string(),
            params,
        });

        // Register pending method for the reader task.
        let pending_tx = server.pending_tx.clone();
        let method_str = method.to_string();
        let uri_clone = Some(uri.clone());
        self.rt.spawn(async move {
            let _ = pending_tx.send((id, method_str, uri_clone)).await;
        });

        // Send the message.
        let tx = server.request_tx.clone();
        self.rt.spawn(async move {
            let _ = tx.send(msg).await;
        });

        Ok(id)
    }

    /// Send a message to the server for a given language.
    fn send_to_server(&self, language_id: &LanguageId, msg: LspMessage) {
        if let Some(server) = self.servers.get(language_id) {
            let tx = server.request_tx.clone();
            self.rt.spawn(async move {
                let _ = tx.send(msg).await;
            });
        }
    }

    /// Send the initialize request to a newly spawned server.
    fn send_initialize(&mut self, language_id: &LanguageId) -> Result<(), String> {
        let server = self
            .servers
            .get_mut(language_id)
            .ok_or("server not found")?;

        let id = RequestId::new(server.next_id);
        server.next_id += 1;

        let root_uri = self.workspace_root.as_ref().map(|u| u.as_str().to_string());

        let params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": client_capabilities(),
            "initializationOptions":
                self.registry
                    .get(language_id)
                    .and_then(|c| c.init_options.clone()),
        });

        let msg = LspMessage::Request(LspRequest {
            id,
            method: "initialize".into(),
            params,
        });

        // Register pending method.
        let pending_tx = server.pending_tx.clone();
        self.rt.spawn(async move {
            let _ = pending_tx.send((id, "initialize".into(), None)).await;
        });

        let tx = server.request_tx.clone();
        self.rt.spawn(async move {
            let _ = tx.send(msg).await;
        });

        Ok(())
    }

    /// Drain pending events from server tasks. Non-blocking.
    pub fn poll_events(&mut self) -> Vec<LspEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Handle a ServerReady event: send initialized notification.
    pub fn handle_server_ready(&mut self, language_id: &LanguageId) {
        if let Some(server) = self.servers.get_mut(language_id) {
            server.state = ServerState::Ready;
        }

        // Send initialized notification.
        let notif = LspMessage::Notification(LspNotification {
            method: "initialized".into(),
            params: serde_json::json!({}),
        });
        self.send_to_server(language_id, notif);
    }
}

/// Client capabilities advertised to the server.
fn client_capabilities() -> serde_json::Value {
    serde_json::json!({
        "textDocument": {
            "synchronization": {
                "dynamicRegistration": false,
                "willSave": false,
                "willSaveWaitUntil": false,
                "didSave": true,
            },
            "completion": {
                "completionItem": {
                    "snippetSupport": false,
                    "commitCharactersSupport": true,
                },
            },
            "hover": {
                "contentFormat": ["plaintext"],
            },
            "definition": {
                "dynamicRegistration": false,
            },
            "references": {
                "dynamicRegistration": false,
            },
            "documentSymbol": {
                "dynamicRegistration": false,
                "hierarchicalDocumentSymbolSupport": true,
            },
            "formatting": {
                "dynamicRegistration": false,
            },
            "rename": {
                "dynamicRegistration": false,
                "prepareSupport": true,
            },
            "publishDiagnostics": {
                "relatedInformation": false,
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    fn test_client(rt: &tokio::runtime::Runtime) -> LspClient {
        LspClient::new(Some("/tmp/test"), rt.handle().clone())
    }

    #[test]
    fn default_registry_has_rust() {
        let rt = test_rt();
        let client = test_client(&rt);
        let lang = client.language_for_extension(".rs");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().as_str(), "rust");
    }

    #[test]
    fn default_registry_has_go() {
        let rt = test_rt();
        let client = test_client(&rt);
        let lang = client.language_for_extension(".go");
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().as_str(), "go");
    }

    #[test]
    fn default_registry_has_typescript() {
        let rt = test_rt();
        let client = test_client(&rt);
        assert_eq!(
            client.language_for_extension(".ts").unwrap().as_str(),
            "typescript"
        );
        assert_eq!(
            client.language_for_extension(".tsx").unwrap().as_str(),
            "typescript"
        );
    }

    #[test]
    fn unknown_extension_returns_none() {
        let rt = test_rt();
        let client = test_client(&rt);
        assert!(client.language_for_extension(".txt").is_none());
        assert!(client.language_for_extension(".md").is_none());
    }

    #[test]
    fn custom_server_config() {
        let rt = test_rt();
        let mut client = test_client(&rt);
        client.register(ServerConfig {
            language_id: LanguageId::new("python"),
            command: "pyright".into(),
            args: vec![],
            extensions: vec![".py".into()],
            init_options: None,
            root_markers: vec!["pyproject.toml".into()],
        });
        assert_eq!(
            client.language_for_extension(".py").unwrap().as_str(),
            "python"
        );
    }

    #[test]
    fn is_ready_false_when_no_server() {
        let rt = test_rt();
        let client = test_client(&rt);
        let lang = LanguageId::new("rust");
        assert!(!client.is_ready(&lang));
    }

    #[test]
    fn poll_events_empty_initially() {
        let rt = test_rt();
        let mut client = test_client(&rt);
        assert!(client.poll_events().is_empty());
    }

    #[test]
    fn language_for_uri_works() {
        let rt = test_rt();
        let client = test_client(&rt);
        let uri = DocumentUri::from_path("/home/user/main.rs");
        let lang = client.language_for_uri(&uri);
        assert!(lang.is_some());
        assert_eq!(lang.unwrap().as_str(), "rust");
    }

    #[test]
    fn language_for_uri_unknown() {
        let rt = test_rt();
        let client = test_client(&rt);
        let uri = DocumentUri::from_path("/home/user/readme.txt");
        assert!(client.language_for_uri(&uri).is_none());
    }

    #[test]
    fn client_capabilities_has_required_fields() {
        let caps = client_capabilities();
        assert!(caps.get("textDocument").is_some());
        let td = &caps["textDocument"];
        assert!(td.get("completion").is_some());
        assert!(td.get("hover").is_some());
        assert!(td.get("definition").is_some());
        assert!(td.get("references").is_some());
        assert!(td.get("documentSymbol").is_some());
        assert!(td.get("formatting").is_some());
        assert!(td.get("rename").is_some());
        assert!(td.get("publishDiagnostics").is_some());
    }
}
