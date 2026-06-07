//! Key specification parsing and formatting.

use txv_core::prelude::*;

/// Parse a keyspec string like "Ctrl-Q" into a KeyEvent.
pub fn parse_keyspec(spec: &str) -> Option<KeyEvent> {
    let parts: Vec<&str> = spec.split('-').collect();
    let mut modifiers = KeyMod::NONE;
    let key_part = parts.last()?;

    for &part in &parts[..parts.len().saturating_sub(1)] {
        match part {
            "Ctrl" => modifiers = modifiers.with_ctrl(),
            "Alt" => modifiers = modifiers.with_alt(),
            "Shift" => modifiers = modifiers.with_shift(),
            _ => {}
        }
    }

    let code = match *key_part {
        "Enter" => KeyCode::Enter,
        "Escape" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        s if s.starts_with('F') => {
            let n: u8 = s[1..].parse().ok()?;
            KeyCode::F(n)
        }
        s if s.len() == 1 => KeyCode::Char(s.chars().next()?),
        _ => return None,
    };

    Some(KeyEvent::new(code, modifiers))
}

/// Format a keyspec into a short label for the status bar.
pub fn format_key_label(spec: &str) -> String {
    let short = spec.replace("Ctrl-", "^").replace("Alt-", "M-").replace("Shift-", "S-");
    if short.len() <= 8 {
        short
    } else {
        String::new()
    }
}

/// Convert a KeyEvent back to a keyspec string.
pub fn key_to_spec(key: KeyEvent) -> String {
    let mut parts = Vec::new();
    if key.modifiers().ctrl() {
        parts.push("Ctrl");
    }
    if key.modifiers().alt() {
        parts.push("Alt");
    }
    if key.modifiers().shift() {
        parts.push("Shift");
    }
    let key_name = match key.code() {
        KeyCode::Char(c) => {
            let upper = c.to_ascii_uppercase();
            return if parts.is_empty() {
                c.to_string()
            } else {
                format!("{}-{upper}", parts.join("-"))
            };
        }
        KeyCode::F(n) => format!("F{n}"),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Escape".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Delete => "Delete".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        _ => return String::new(),
    };
    if parts.is_empty() {
        key_name
    } else {
        format!("{}-{key_name}", parts.join("-"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ctrl_q() {
        let key = parse_keyspec("Ctrl-Q");
        assert_eq!(key, Some(KeyEvent::new(KeyCode::Char('Q'), KeyMod::CTRL,)));
    }

    #[test]
    fn parse_f1() {
        let key = parse_keyspec("F1");
        assert_eq!(key, Some(KeyEvent::new(KeyCode::F(1), KeyMod::NONE)));
    }

    #[test]
    fn parse_escape() {
        let key = parse_keyspec("Escape");
        assert_eq!(key, Some(KeyEvent::new(KeyCode::Esc, KeyMod::NONE)));
    }

    #[test]
    fn to_spec_simple_char() {
        let key = KeyEvent::new(KeyCode::Char('a'), KeyMod::NONE);
        assert_eq!(key_to_spec(key), "a");
    }

    #[test]
    fn to_spec_ctrl_q() {
        let key = KeyEvent::new(KeyCode::Char('q'), KeyMod::CTRL);
        assert_eq!(key_to_spec(key), "Ctrl-Q");
    }
}
