//! Hook registry — stores event hooks with optional compiled filters.

use regex::Regex;

/// Events that can trigger hooks.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookEvent {
    FileSave,
    FileOpen,
    FileClose,
    BuildDone,
    TabSwitched,
    Startup,
    CharInserted,
    CharDeleted,
    WordCompleted,
    Idle,
    Paste,
    ModeChanged,
    SelectionChanged,
    LspStart,
}

impl HookEvent {
    pub fn parse_name(s: &str) -> Option<Self> {
        match s {
            "file-save" => Some(Self::FileSave),
            "file-open" => Some(Self::FileOpen),
            "file-close" => Some(Self::FileClose),
            "build-done" => Some(Self::BuildDone),
            "tab-switched" => Some(Self::TabSwitched),
            "startup" => Some(Self::Startup),
            "char-inserted" => Some(Self::CharInserted),
            "char-deleted" => Some(Self::CharDeleted),
            "word-completed" => Some(Self::WordCompleted),
            "idle" => Some(Self::Idle),
            "paste" => Some(Self::Paste),
            "mode-changed" => Some(Self::ModeChanged),
            "selection-changed" => Some(Self::SelectionChanged),
            "lsp-start" => Some(Self::LspStart),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FileSave => "file-save",
            Self::FileOpen => "file-open",
            Self::FileClose => "file-close",
            Self::BuildDone => "build-done",
            Self::TabSwitched => "tab-switched",
            Self::Startup => "startup",
            Self::CharInserted => "char-inserted",
            Self::CharDeleted => "char-deleted",
            Self::WordCompleted => "word-completed",
            Self::Idle => "idle",
            Self::Paste => "paste",
            Self::ModeChanged => "mode-changed",
            Self::SelectionChanged => "selection-changed",
            Self::LspStart => "lsp-start",
        }
    }
}

/// A compiled filter for hook matching.
pub enum CompiledFilter {
    Regex(Regex),
    Millis(u64),
}

/// A single registered hook.
pub struct Hook {
    pub event: HookEvent,
    pub filter: Option<CompiledFilter>,
    pub body: String,
}

/// Registry of all hooks, fired in declaration order.
#[derive(Default)]
pub struct HookRegistry {
    hooks: Vec<Hook>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Add a hook. Filter is compiled as regex for char/word events, millis for idle.
    pub fn add(&mut self, event: HookEvent, filter: Option<&str>, body: String) -> Result<(), String> {
        let compiled = match filter {
            None => None,
            Some(f) => match &event {
                HookEvent::Idle => {
                    let ms = f.parse::<u64>().map_err(|e| format!("invalid idle ms: {e}"))?;
                    Some(CompiledFilter::Millis(ms))
                }
                _ => {
                    let re = Regex::new(f).map_err(|e| format!("invalid filter regex: {e}"))?;
                    Some(CompiledFilter::Regex(re))
                }
            },
        };
        self.hooks.push(Hook {
            event,
            filter: compiled,
            body,
        });
        Ok(())
    }

    /// Remove all hooks for a given event.
    pub fn remove(&mut self, event: &HookEvent) {
        self.hooks.retain(|h| &h.event != event);
    }

    /// List hooks, optionally filtered by event.
    pub fn list(&self, event: Option<&HookEvent>) -> Vec<String> {
        self.hooks
            .iter()
            .filter(|h| event.is_none() || event == Some(&h.event))
            .map(|h| format!("{}: {}", h.event.as_str(), h.body))
            .collect()
    }

    /// Fire hooks for an event with context. Returns scripts to execute.
    pub fn fire(&self, event: &HookEvent, context: &str) -> Vec<String> {
        self.hooks
            .iter()
            .filter(|h| &h.event == event)
            .filter(|h| matches_filter(&h.filter, context))
            .map(|h| h.body.clone())
            .collect()
    }
}

fn matches_filter(filter: &Option<CompiledFilter>, context: &str) -> bool {
    match filter {
        None => true,
        Some(CompiledFilter::Regex(re)) => re.is_match(context),
        Some(CompiledFilter::Millis(_)) => true, // Caller checks timing
    }
}

/// Trigger event pushed by the editor for the handler to fire.
#[derive(Debug, Clone)]
pub enum HookTrigger {
    CharInserted(char),
    CharDeleted(char),
    WordCompleted(String),
}
