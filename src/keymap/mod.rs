// Hotkey definitions and dispatch.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Application-level actions triggered by hotkeys.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    LaunchEditor,
    OpenSearch,
    DiffCurrentFile,
    GitLog,
    SaveSession,
    LoadSession,
    /// Key not mapped at app level — forward to focused panel.
    Forward(KeyEvent),
}

/// Map a key event to an app-level action.
pub fn map_key(key: KeyEvent) -> Action {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    match (ctrl, alt, shift, key.code) {
        // App-level
        (true, false, false, KeyCode::Char('q')) => Action::Quit,
        (true, false, false, KeyCode::Char('l')) => Action::RotateLayout,
        (true, false, false, KeyCode::Char('b')) => Action::ToggleTree,
        (true, false, false, KeyCode::Tab) => Action::CycleFocus,
        (true, false, false, KeyCode::Char('o')) => Action::TogglePinOutput,
        (true, false, false, KeyCode::Char('e')) => Action::LaunchEditor,
        (true, false, false, KeyCode::Char('p')) => Action::OpenSearch,
        (true, false, false, KeyCode::Char('d')) => Action::DiffCurrentFile,
        (true, false, false, KeyCode::Char('g')) => Action::GitLog,
        (true, false, true, KeyCode::Char('S')) => Action::SaveSession,
        (true, false, true, KeyCode::Char('O')) => Action::LoadSession,

        // Tab management
        (true, false, false, KeyCode::Char('k')) => Action::NewKiroTab,
        (true, false, false, KeyCode::Char('s')) => Action::NewShellTab,
        (true, false, false, KeyCode::Char('w')) => Action::CloseTab,

        // Alt (no shift) = tab switching
        (false, true, false, KeyCode::Left) => Action::PrevTab,
        (false, true, false, KeyCode::Right) => Action::NextTab,

        // Ctrl+Alt = resize by 1
        (true, true, false, KeyCode::Left) => Action::ResizeTree(-1),
        (true, true, false, KeyCode::Right) => Action::ResizeTree(1),
        (true, true, false, KeyCode::Up) => Action::ResizeInteractive(1),
        (true, true, false, KeyCode::Down) => Action::ResizeInteractive(-1),

        // Alt+Shift = resize by 5
        (false, true, true, KeyCode::Left) => Action::ResizeTree(-5),
        (false, true, true, KeyCode::Right) => Action::ResizeTree(5),
        (false, true, true, KeyCode::Up) => Action::ResizeInteractive(5),
        (false, true, true, KeyCode::Down) => Action::ResizeInteractive(-5),

        _ => Action::Forward(key),
    }
}
