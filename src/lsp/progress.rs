//! LSP progress tracking — parses $/progress notifications and tracks per-language state.

use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value;

/// State of an LSP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspServerState {
    Starting,
    Indexing {
        percent: Option<u8>,
        message: Option<String>,
    },
    Ready,
    Error,
}

/// Tracks LSP server states per language.
pub struct LspStatusTracker {
    servers: HashMap<String, LspServerState>,
    started_at: HashMap<String, Instant>,
}

impl Default for LspStatusTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LspStatusTracker {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
            started_at: HashMap::new(),
        }
    }

    pub fn set_state(&mut self, lang: &str, state: LspServerState) {
        if state == LspServerState::Starting {
            self.started_at.entry(lang.to_string()).or_insert_with(Instant::now);
        } else {
            self.started_at.remove(lang);
        }
        self.servers.insert(lang.to_string(), state);
    }

    pub fn get(&self, lang: &str) -> Option<&LspServerState> {
        self.servers.get(lang)
    }

    pub fn remove(&mut self, lang: &str) {
        self.servers.remove(lang);
        self.started_at.remove(lang);
    }

    /// Returns true if any server is in Starting state.
    pub fn has_starting(&self) -> bool {
        self.servers.values().any(|s| *s == LspServerState::Starting)
    }

    pub fn snapshot(&self) -> Vec<(String, LspServerState, Option<u64>)> {
        let mut items: Vec<_> = self
            .servers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone(), self.elapsed_secs(k)))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        items
    }

    /// Get elapsed seconds since a language entered Starting state.
    pub fn elapsed_secs(&self, lang: &str) -> Option<u64> {
        self.started_at.get(lang).map(|t| t.elapsed().as_secs())
    }

    /// Handle a $/progress notification. Returns true if state changed.
    pub fn handle_progress(&mut self, lang: &str, params: &Value) -> bool {
        let token = params
            .get("token")
            .and_then(|t| t.as_str().or_else(|| t.as_u64().map(|_| "")));
        if token.is_none() {
            return false;
        }
        let Some(value) = params.get("value") else {
            return false;
        };
        let kind = value.get("kind").and_then(|k| k.as_str()).unwrap_or("");
        match kind {
            "begin" => self.handle_progress_begin(lang, value),
            "report" => self.handle_progress_report(lang, value),
            "end" => self.handle_progress_end(lang),
            _ => false,
        }
    }

    fn handle_progress_begin(&mut self, lang: &str, value: &Value) -> bool {
        let msg = value.get("message").and_then(|m| m.as_str()).map(|s| s.to_string());
        let pct = value.get("percentage").and_then(|p| p.as_u64()).map(|p| p as u8);
        self.servers.insert(
            lang.to_string(),
            LspServerState::Indexing {
                percent: pct,
                message: msg,
            },
        );
        true
    }

    fn handle_progress_report(&mut self, lang: &str, value: &Value) -> bool {
        let pct = value.get("percentage").and_then(|p| p.as_u64()).map(|p| p as u8);
        let msg = value.get("message").and_then(|m| m.as_str()).map(|s| s.to_string());
        if let Some(LspServerState::Indexing { percent, message }) = self.servers.get_mut(lang) {
            if pct.is_some() {
                *percent = pct;
            }
            if msg.is_some() {
                *message = msg;
            }
            true
        } else {
            false
        }
    }

    fn handle_progress_end(&mut self, lang: &str) -> bool {
        if self.servers.get(lang) == Some(&LspServerState::Error) {
            return false;
        }
        self.servers.insert(lang.to_string(), LspServerState::Ready);
        true
    }
}

/// Format a compact label for the status bar from a state snapshot.
pub fn format_status_label(snapshot: &[(String, LspServerState, Option<u64>)]) -> String {
    if snapshot.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = snapshot
        .iter()
        .map(|(lang, state, elapsed)| {
            let short = short_name(lang);
            match state {
                LspServerState::Starting => {
                    if let Some(secs) = elapsed.filter(|&s| s >= 3) {
                        format!("{short} …{secs}s")
                    } else {
                        format!("{short} …")
                    }
                }
                LspServerState::Indexing { percent: Some(p), .. } => format!("{short} {p}%"),
                LspServerState::Indexing { .. } => format!("{short} ⟳"),
                LspServerState::Ready => format!("{short} ✓"),
                LspServerState::Error => format!("{short} ✗"),
            }
        })
        .collect();
    parts.join(" ")
}

fn short_name(lang: &str) -> &str {
    match lang {
        "typescript" => "ts",
        "javascript" => "js",
        "python" => "py",
        "cpp" => "c++",
        _ => lang,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_transitions() {
        let mut tracker = LspStatusTracker::new();
        tracker.set_state("rust", LspServerState::Starting);
        let snap = tracker.snapshot();
        assert_eq!(snap[0].0, "rust");
        assert_eq!(snap[0].1, LspServerState::Starting);

        tracker.set_state("rust", LspServerState::Ready);
        let snap = tracker.snapshot();
        assert_eq!(snap[0].1, LspServerState::Ready);
    }

    #[test]
    fn progress_begin_report_end() {
        let mut tracker = LspStatusTracker::new();
        tracker.set_state("rust", LspServerState::Ready);

        let begin = serde_json::json!({
            "token": "rustAnalyzer/Indexing",
            "value": {"kind": "begin", "title": "Indexing", "percentage": 0}
        });
        assert!(tracker.handle_progress("rust", &begin));
        assert!(matches!(
            tracker.servers.get("rust"),
            Some(LspServerState::Indexing { .. })
        ));

        let report =
            serde_json::json!({"token": "rustAnalyzer/Indexing", "value": {"kind": "report", "percentage": 50}});
        assert!(tracker.handle_progress("rust", &report));
        if let Some(LspServerState::Indexing { percent, .. }) = tracker.servers.get("rust") {
            assert_eq!(*percent, Some(50));
        } else {
            panic!("expected Indexing");
        }

        let end = serde_json::json!({"token": "rustAnalyzer/Indexing", "value": {"kind": "end"}});
        assert!(tracker.handle_progress("rust", &end));
        assert_eq!(tracker.servers.get("rust"), Some(&LspServerState::Ready));
    }

    #[test]
    fn format_label_multiple_languages() {
        let snapshot = vec![
            ("go".into(), LspServerState::Ready, None),
            (
                "rust".into(),
                LspServerState::Indexing {
                    percent: Some(42),
                    message: None,
                },
                None,
            ),
        ];
        let label = format_status_label(&snapshot);
        assert!(label.contains("go"));
        assert!(label.contains("rust 42%"));
    }

    #[test]
    fn format_label_empty() {
        assert_eq!(format_status_label(&[]), "");
    }

    #[test]
    fn short_names() {
        assert_eq!(short_name("typescript"), "ts");
        assert_eq!(short_name("rust"), "rust");
        assert_eq!(short_name("python"), "py");
    }
}
