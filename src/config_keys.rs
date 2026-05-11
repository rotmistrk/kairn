//! Key binding parsing from config variable values.

use txv_core::prelude::*;

/// Parse a simple key variable value into a KeyEvent.
/// Supports: single chars ("s"), modifier combos ("Ctrl-s"), function keys ("F1").
pub(crate) fn parse_key_var(spec: &str) -> Option<KeyEvent> {
    let spec = spec.trim();
    if spec.is_empty() {
        return None;
    }
    let parts: Vec<&str> = spec.split('-').collect();
    let mut modifiers = KeyMod::default();
    let key_part = parts.last()?;
    for &part in &parts[..parts.len().saturating_sub(1)] {
        match part {
            "Ctrl" | "ctrl" => modifiers.ctrl = true,
            "Alt" | "alt" => modifiers.alt = true,
            "Shift" | "shift" => modifiers.shift = true,
            _ => {}
        }
    }
    let code = parse_key_code(key_part)?;
    Some(KeyEvent { code, modifiers })
}

fn parse_key_code(s: &str) -> Option<KeyCode> {
    if s.len() == 1 {
        return Some(KeyCode::Char(s.chars().next()?));
    }
    if let Some(n) = s.strip_prefix('F').or_else(|| s.strip_prefix('f')) {
        if let Ok(num) = n.parse::<u8>() {
            return Some(KeyCode::F(num));
        }
    }
    match s {
        "Esc" | "esc" => Some(KeyCode::Esc),
        "Enter" | "enter" => Some(KeyCode::Enter),
        "Tab" | "tab" => Some(KeyCode::Tab),
        "Left" | "left" => Some(KeyCode::Left),
        "Right" | "right" => Some(KeyCode::Right),
        "Up" | "up" => Some(KeyCode::Up),
        "Down" | "down" => Some(KeyCode::Down),
        _ => None,
    }
}
