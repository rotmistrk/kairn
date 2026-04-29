// Data-driven keymap: parses key combos from config strings.
// Supports single-chord ("ctrl+q") and two-chord sequences ("ctrl+x k").

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::Config;

/// Application-level actions triggered by hotkeys.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Quit,
    RotateLayout,
    ToggleTree,
    CycleFocus,
    FocusTree,
    FocusMain,
    FocusTerminal,
    ResizeTree(i16),
    ResizeInteractive(i16),
    NewKiroTab,
    NewShellTab,
    NextTab,
    PrevTab,
    CloseTab,
    TogglePinOutput,
    PeekScreen,
    LaunchEditor,
    SuspendToShell,
    OpenSearch,
    DiffCurrentFile,
    GitLog,
    ShowHelp,
    SaveSession,
    LoadSession,
    ScrollUp,
    ScrollDown,
    ScrollTop,
    ScrollBottom,
    CycleModeNext,
    CycleModePrev,
    ToggleLeftPanel,
    Redraw,
    RefreshTree,
    CaptureAll,
    CaptureOutput,
    SaveBuffer,
    /// Key not mapped at app level — forward to focused panel.
    Forward(KeyEvent),
}

type KeyChord = (KeyModifiers, KeyCode);

/// Result of feeding a key event into the keymap.
#[derive(Debug, Clone)]
pub enum MapResult {
    /// Resolved to a concrete action.
    Action(Action),
    /// First chord of a two-key sequence matched; waiting for second.
    Pending,
}

impl Action {
    /// Actions that should be intercepted even when the terminal
    /// panel is focused (F-keys, Ctrl+Shift combos, quit, tab
    /// management, resize, etc.).  Everything else is forwarded
    /// to the PTY so readline editing keys work.
    pub fn is_global(&self) -> bool {
        matches!(
            self,
            Action::Quit
                | Action::ShowHelp
                | Action::FocusTree
                | Action::FocusMain
                | Action::FocusTerminal
                | Action::CycleFocus
                | Action::ToggleLeftPanel
                | Action::Redraw
                | Action::ResizeTree(_)
                | Action::ResizeInteractive(_)
                | Action::CycleModeNext
                | Action::CycleModePrev
                | Action::ScrollUp
                | Action::ScrollDown
                | Action::ScrollTop
                | Action::ScrollBottom
                | Action::NewKiroTab
                | Action::NewShellTab
                | Action::CloseTab
                | Action::NextTab
                | Action::PrevTab
                | Action::SaveSession
                | Action::LoadSession
                | Action::RefreshTree
                | Action::CaptureAll
                | Action::CaptureOutput
                | Action::SaveBuffer
        )
    }
}

/// Parsed keymap built from config at startup.
pub struct Keymap {
    /// Single-chord bindings: one key → action.
    single: HashMap<KeyChord, Action>,
    /// Two-chord bindings: (prefix, second) → action.
    seq: HashMap<(KeyChord, KeyChord), Action>,
    /// Set of known prefix chords (derived from `seq` keys).
    prefixes: std::collections::HashSet<KeyChord>,
    /// Pending prefix chord, if any.
    pending: Option<KeyChord>,
}

impl Keymap {
    /// Build keymap from config keybindings.
    pub fn from_config(config: &Config) -> Self {
        let mut single = HashMap::new();
        let mut seq = HashMap::new();
        for (action_name, combo) in &config.keys {
            let Some(action) = name_to_action(action_name) else {
                continue;
            };
            let chords = parse_binding(&combo.0);
            match chords.as_slice() {
                [one] => {
                    single.insert(*one, action);
                }
                [first, second] => {
                    seq.insert((*first, *second), action);
                }
                _ => {}
            }
        }
        let prefixes = seq.keys().map(|(p, _)| *p).collect();
        Self {
            single,
            seq,
            prefixes,
            pending: None,
        }
    }

    /// Feed a key event. Returns the mapping result.
    pub fn map_key(&mut self, key: KeyEvent) -> MapResult {
        let chord = normalize_chord(key);
        if let Some(prefix) = self.pending.take() {
            if let Some(action) = self.seq.get(&(prefix, chord)) {
                return MapResult::Action(action.clone());
            }
            // Second key didn't match — discard prefix, forward key.
            return MapResult::Action(Action::Forward(key));
        }
        // Check if this chord starts a sequence.
        if self.prefixes.contains(&chord) {
            self.pending = Some(chord);
            return MapResult::Pending;
        }
        if let Some(action) = self.single.get(&chord) {
            return MapResult::Action(action.clone());
        }
        MapResult::Action(Action::Forward(key))
    }

    /// Whether a prefix chord is pending (for status bar display).
    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Cancel any pending prefix.
    pub fn cancel_pending(&mut self) {
        self.pending = None;
    }

    /// Human-readable label for the pending prefix, e.g. "C-x".
    pub fn pending_label(&self) -> Option<String> {
        self.pending.map(|(mods, code)| format_chord(mods, code))
    }
}

fn normalize_chord(key: KeyEvent) -> KeyChord {
    let mods = key.modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
    (mods, key.code)
}

fn format_chord(mods: KeyModifiers, code: KeyCode) -> String {
    let mut s = String::new();
    if mods.contains(KeyModifiers::CONTROL) {
        s.push_str("C-");
    }
    if mods.contains(KeyModifiers::ALT) {
        s.push_str("M-");
    }
    if mods.contains(KeyModifiers::SHIFT) {
        s.push_str("S-");
    }
    match code {
        KeyCode::Char(c) => s.push(c),
        KeyCode::F(n) => s.push_str(&format!("F{n}")),
        _ => s.push_str(&format!("{code:?}")),
    }
    s
}

fn name_to_action(name: &str) -> Option<Action> {
    name_to_global_action(name).or_else(|| name_to_tab_action(name))
}

fn name_to_global_action(name: &str) -> Option<Action> {
    Some(match name {
        "quit" => Action::Quit,
        "rotate_layout" => Action::RotateLayout,
        "toggle_tree" => Action::ToggleTree,
        "cycle_focus" => Action::CycleFocus,
        "focus_tree" => Action::FocusTree,
        "focus_main" => Action::FocusMain,
        "focus_terminal" => Action::FocusTerminal,
        "peek_screen" => Action::PeekScreen,
        "launch_editor" => Action::LaunchEditor,
        "suspend_to_shell" => Action::SuspendToShell,
        "open_search" => Action::OpenSearch,
        "diff_current_file" => Action::DiffCurrentFile,
        "git_log" => Action::GitLog,
        "show_help" => Action::ShowHelp,
        "save_session" => Action::SaveSession,
        "load_session" => Action::LoadSession,
        "scroll_up" => Action::ScrollUp,
        "scroll_down" => Action::ScrollDown,
        "scroll_top" => Action::ScrollTop,
        "scroll_bottom" => Action::ScrollBottom,
        "cycle_mode_next" => Action::CycleModeNext,
        "cycle_mode_prev" => Action::CycleModePrev,
        "toggle_left_panel" => Action::ToggleLeftPanel,
        "redraw" => Action::Redraw,
        "refresh_tree" => Action::RefreshTree,
        "capture_all" => Action::CaptureAll,
        "capture_output" => Action::CaptureOutput,
        "save_buffer" => Action::SaveBuffer,
        _ => return None,
    })
}

fn name_to_tab_action(name: &str) -> Option<Action> {
    Some(match name {
        "new_kiro_tab" => Action::NewKiroTab,
        "new_shell_tab" => Action::NewShellTab,
        "close_tab" => Action::CloseTab,
        "prev_tab" => Action::PrevTab,
        "next_tab" => Action::NextTab,
        "resize_tree_shrink" => Action::ResizeTree(-1),
        "resize_tree_grow" => Action::ResizeTree(1),
        "resize_interactive_grow" => Action::ResizeInteractive(1),
        "resize_interactive_shrink" => Action::ResizeInteractive(-1),
        "resize_tree_shrink5" => Action::ResizeTree(-5),
        "resize_tree_grow5" => Action::ResizeTree(5),
        "resize_interactive_grow5" => Action::ResizeInteractive(5),
        "resize_interactive_shrink5" => Action::ResizeInteractive(-5),
        _ => return None,
    })
}

/// Parse a binding string into one or two chords.
/// Single: "ctrl+q"  →  [(Ctrl, Q)]
/// Sequence: "ctrl+x k"  →  [(Ctrl, X), (None, K)]
fn parse_binding(s: &str) -> Vec<KeyChord> {
    s.split_whitespace().filter_map(parse_combo).collect()
}

/// Parse a single combo string like "ctrl+shift+s" into (KeyModifiers, KeyCode).
fn parse_combo(s: &str) -> Option<KeyChord> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut mods = KeyModifiers::empty();
    let mut key_part = "";

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" => mods |= KeyModifiers::CONTROL,
            "alt" => mods |= KeyModifiers::ALT,
            "shift" => mods |= KeyModifiers::SHIFT,
            _ => key_part = part,
        }
    }

    let code = parse_key_code(key_part)?;
    Some((mods, code))
}

fn parse_key_code(s: &str) -> Option<KeyCode> {
    Some(match s.to_lowercase().as_str() {
        "tab" => KeyCode::Tab,
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdn" => KeyCode::PageDown,
        "f1" => KeyCode::F(1),
        "f2" => KeyCode::F(2),
        "f3" => KeyCode::F(3),
        "f4" => KeyCode::F(4),
        "f5" => KeyCode::F(5),
        "f6" => KeyCode::F(6),
        "f7" => KeyCode::F(7),
        "f8" => KeyCode::F(8),
        "f9" => KeyCode::F(9),
        "f10" => KeyCode::F(10),
        "f11" => KeyCode::F(11),
        "f12" => KeyCode::F(12),
        "space" => KeyCode::Char(' '),
        "/" => KeyCode::Char('/'),
        s if s.len() == 1 => {
            let c = s.chars().next()?;
            KeyCode::Char(c)
        }
        _ => return None,
    })
}
