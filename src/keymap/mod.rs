// Data-driven keymap: parses key combos from config strings.

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
    /// Key not mapped at app level — forward to focused panel.
    Forward(KeyEvent),
}

/// Parsed keymap built from config at startup.
pub struct Keymap {
    bindings: HashMap<(KeyModifiers, KeyCode), Action>,
}

impl Keymap {
    /// Build keymap from config keybindings.
    pub fn from_config(config: &Config) -> Self {
        let mut bindings = HashMap::new();
        for (action_name, combo) in &config.keys {
            if let Some(action) = name_to_action(action_name) {
                if let Some((mods, code)) = parse_combo(&combo.0) {
                    bindings.insert((mods, code), action);
                }
            }
        }
        Self { bindings }
    }

    /// Map a key event to an action.
    pub fn map_key(&self, key: KeyEvent) -> Action {
        let lookup_mods =
            key.modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
        self.bindings
            .get(&(lookup_mods, key.code))
            .cloned()
            .unwrap_or(Action::Forward(key))
    }
}

fn name_to_action(name: &str) -> Option<Action> {
    Some(match name {
        "quit" => Action::Quit,
        "rotate_layout" => Action::RotateLayout,
        "toggle_tree" => Action::ToggleTree,
        "cycle_focus" => Action::CycleFocus,
        "toggle_pin_output" => Action::TogglePinOutput,
        "peek_screen" => Action::PeekScreen,
        "launch_editor" => Action::LaunchEditor,
        "suspend_to_shell" => Action::SuspendToShell,
        "open_search" => Action::OpenSearch,
        "diff_current_file" => Action::DiffCurrentFile,
        "git_log" => Action::GitLog,
        "show_help" => Action::ShowHelp,
        "save_session" => Action::SaveSession,
        "load_session" => Action::LoadSession,
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

/// Parse a combo string like "ctrl+shift+s" into (KeyModifiers, KeyCode).
fn parse_combo(s: &str) -> Option<(KeyModifiers, KeyCode)> {
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
