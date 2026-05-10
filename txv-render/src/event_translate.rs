//! Crossterm event translation — converts crossterm events to txv-core events.

use crossterm::event as ct_event;
use txv_core::event::{Event, KeyCode, KeyEvent, KeyMod, MouseAction, MouseButton, MouseEvent};

pub(crate) fn translate_key(key: ct_event::KeyEvent) -> Option<Event> {
    // Ignore key release events
    if key.kind == ct_event::KeyEventKind::Release {
        return None;
    }

    let modifiers = KeyMod {
        ctrl: key.modifiers.contains(ct_event::KeyModifiers::CONTROL),
        alt: key.modifiers.contains(ct_event::KeyModifiers::ALT),
        shift: key.modifiers.contains(ct_event::KeyModifiers::SHIFT),
    };

    let code = match key.code {
        ct_event::KeyCode::Char(c) => KeyCode::Char(c),
        ct_event::KeyCode::F(n) => KeyCode::F(n),
        ct_event::KeyCode::Enter => KeyCode::Enter,
        ct_event::KeyCode::Esc => KeyCode::Esc,
        ct_event::KeyCode::Tab => KeyCode::Tab,
        ct_event::KeyCode::BackTab => KeyCode::BackTab,
        ct_event::KeyCode::Backspace => KeyCode::Backspace,
        ct_event::KeyCode::Delete => KeyCode::Delete,
        ct_event::KeyCode::Left => KeyCode::Left,
        ct_event::KeyCode::Right => KeyCode::Right,
        ct_event::KeyCode::Up => KeyCode::Up,
        ct_event::KeyCode::Down => KeyCode::Down,
        ct_event::KeyCode::Home => KeyCode::Home,
        ct_event::KeyCode::End => KeyCode::End,
        ct_event::KeyCode::PageUp => KeyCode::PageUp,
        ct_event::KeyCode::PageDown => KeyCode::PageDown,
        ct_event::KeyCode::Insert => KeyCode::Insert,
        _ => return None,
    };

    Some(Event::Key(KeyEvent { code, modifiers }))
}

pub(crate) fn translate_mouse(m: ct_event::MouseEvent) -> Option<Event> {
    let modifiers = KeyMod {
        ctrl: m.modifiers.contains(ct_event::KeyModifiers::CONTROL),
        alt: m.modifiers.contains(ct_event::KeyModifiers::ALT),
        shift: m.modifiers.contains(ct_event::KeyModifiers::SHIFT),
    };

    let action = match m.kind {
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Left) => MouseAction::Press(MouseButton::Left),
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Right) => MouseAction::Press(MouseButton::Right),
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Middle) => MouseAction::Press(MouseButton::Middle),
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Left) => MouseAction::Release(MouseButton::Left),
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Right) => MouseAction::Release(MouseButton::Right),
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Middle) => MouseAction::Release(MouseButton::Middle),
        ct_event::MouseEventKind::Moved | ct_event::MouseEventKind::Drag(_) => MouseAction::Move,
        ct_event::MouseEventKind::ScrollUp => MouseAction::ScrollUp,
        ct_event::MouseEventKind::ScrollDown => MouseAction::ScrollDown,
        _ => return None,
    };

    Some(Event::Mouse(MouseEvent {
        x: m.column,
        y: m.row,
        action,
        modifiers,
    }))
}
