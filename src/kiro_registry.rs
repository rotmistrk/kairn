//! KiroTabRegistry — tracks kiro tab metadata for session persistence.

use std::collections::HashMap;

use crate::session::schema::KiroSessionState;

/// Metadata for a single kiro tab.
#[derive(Debug, Clone)]
pub struct KiroSession {
    pub(crate) display_name: String,
    pub(crate) session_id: Option<String>,
}

/// Registry of active kiro tabs, keyed by tab title (e.g. "Kiro:0").
#[derive(Debug, Default)]
pub struct KiroTabRegistry {
    sessions: HashMap<String, KiroSession>,
}

impl KiroTabRegistry {
    pub fn register(&mut self, name: &str) {
        self.sessions.insert(
            name.to_string(),
            KiroSession {
                display_name: name.to_string(),
                session_id: None,
            },
        );
    }

    pub fn register_with_id(&mut self, name: &str, session_id: Option<String>) {
        self.sessions.insert(
            name.to_string(),
            KiroSession {
                display_name: name.to_string(),
                session_id,
            },
        );
    }

    pub fn remove(&mut self, name: &str) {
        self.sessions.remove(name);
    }

    pub fn rename(&mut self, old: &str, new: &str) {
        if let Some(mut session) = self.sessions.remove(old) {
            session.display_name = new.to_string();
            self.sessions.insert(new.to_string(), session);
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.sessions.contains_key(name)
    }

    pub fn to_state(&self) -> Vec<KiroSessionState> {
        self.sessions
            .values()
            .map(|s| KiroSessionState {
                name: s.display_name.clone(),
                session_id: s.session_id.clone(),
            })
            .collect()
    }

    pub fn from_state(states: &[KiroSessionState]) -> Self {
        let sessions = states
            .iter()
            .map(|s| {
                (
                    s.name.clone(),
                    KiroSession {
                        display_name: s.name.clone(),
                        session_id: s.session_id.clone(),
                    },
                )
            })
            .collect();
        Self { sessions }
    }
}
