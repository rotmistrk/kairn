//! Key-to-bytes encoding for xterm-compatible PTY input.

use txv_core::event::{KeyCode, KeyEvent};

/// Encode a KeyEvent into bytes suitable for writing to a PTY.
pub fn key_to_bytes(key: &KeyEvent) -> Option<Vec<u8>> {
    let bytes = encode_key_code(&key.code, key.modifiers.ctrl);
    if bytes.is_empty() {
        return None;
    }
    if key.modifiers.alt {
        let mut v = vec![0x1b];
        v.extend(&bytes);
        Some(v)
    } else {
        Some(bytes)
    }
}

fn encode_key_code(code: &KeyCode, ctrl: bool) -> Vec<u8> {
    match code {
        KeyCode::Char(c) => encode_char(*c, ctrl),
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => vec![0x1b, b'[', b'Z'],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => vec![0x1b, b'[', b'A'],
        KeyCode::Down => vec![0x1b, b'[', b'B'],
        KeyCode::Right => vec![0x1b, b'[', b'C'],
        KeyCode::Left => vec![0x1b, b'[', b'D'],
        KeyCode::Home => vec![0x1b, b'[', b'H'],
        KeyCode::End => vec![0x1b, b'[', b'F'],
        KeyCode::PageUp => vec![0x1b, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![0x1b, b'[', b'6', b'~'],
        KeyCode::Insert => vec![0x1b, b'[', b'2', b'~'],
        KeyCode::Delete => vec![0x1b, b'[', b'3', b'~'],
        KeyCode::F(n) => f_key_bytes(*n),
    }
}

fn encode_char(ch: char, ctrl: bool) -> Vec<u8> {
    if ctrl && ch.is_ascii_alphabetic() {
        vec![(ch.to_ascii_lowercase() as u8) - b'a' + 1]
    } else {
        let mut buf = [0u8; 4];
        ch.encode_utf8(&mut buf).as_bytes().to_vec()
    }
}

fn f_key_bytes(n: u8) -> Vec<u8> {
    match n {
        1 => vec![0x1b, b'O', b'P'],
        2 => vec![0x1b, b'O', b'Q'],
        3 => vec![0x1b, b'O', b'R'],
        4 => vec![0x1b, b'O', b'S'],
        5 => vec![0x1b, b'[', b'1', b'5', b'~'],
        6 => vec![0x1b, b'[', b'1', b'7', b'~'],
        7 => vec![0x1b, b'[', b'1', b'8', b'~'],
        8 => vec![0x1b, b'[', b'1', b'9', b'~'],
        9 => vec![0x1b, b'[', b'2', b'0', b'~'],
        10 => vec![0x1b, b'[', b'2', b'1', b'~'],
        11 => vec![0x1b, b'[', b'2', b'3', b'~'],
        12 => vec![0x1b, b'[', b'2', b'4', b'~'],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use txv_core::event::KeyMod;

    #[test]
    fn enter_is_cr() {
        let key = KeyEvent { code: KeyCode::Enter, modifiers: KeyMod::default() };
        assert_eq!(key_to_bytes(&key), Some(vec![b'\r']));
    }

    #[test]
    fn backspace_is_del() {
        let key = KeyEvent { code: KeyCode::Backspace, modifiers: KeyMod::default() };
        assert_eq!(key_to_bytes(&key), Some(vec![0x7f]));
    }

    #[test]
    fn arrows_encode() {
        let up = KeyEvent { code: KeyCode::Up, modifiers: KeyMod::default() };
        assert_eq!(key_to_bytes(&up), Some(vec![0x1b, b'[', b'A']));
        let down = KeyEvent { code: KeyCode::Down, modifiers: KeyMod::default() };
        assert_eq!(key_to_bytes(&down), Some(vec![0x1b, b'[', b'B']));
    }

    #[test]
    fn ctrl_c() {
        let key = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyMod { ctrl: true, alt: false, shift: false },
        };
        assert_eq!(key_to_bytes(&key), Some(vec![3]));
    }

    #[test]
    fn char_encoding() {
        let key = KeyEvent { code: KeyCode::Char('a'), modifiers: KeyMod::default() };
        assert_eq!(key_to_bytes(&key), Some(vec![b'a']));
    }
}
