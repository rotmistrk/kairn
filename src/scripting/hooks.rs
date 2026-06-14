//! Hook registry — stores event hooks with optional compiled filters.

/// Events that can trigger hooks.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HookEvent {
    PreSave,
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
            "pre-save" => Some(Self::PreSave),
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
            Self::PreSave => "pre-save",
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
    Pattern(String),
    Millis(u64),
}

/// A single registered hook.
pub struct Hook {
    pub(crate) event: HookEvent,
    pub(crate) filter: Option<CompiledFilter>,
    pub(crate) body: String,
}

pub use super::hook_registry::HookRegistry;

/// Trigger event pushed by the editor for the handler to fire.
#[derive(Debug, Clone)]
pub enum HookTrigger {
    CharInserted(char),
    CharDeleted(char),
    WordCompleted(String),
}
