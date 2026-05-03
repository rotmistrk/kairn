//! Key specification parsing and binding table.
//!
//! Parses key specs like `"ctrl+s"` and `"ctrl+x ctrl+s"` into
//! structured [`KeySpec`] values. [`BindingTable`] stores and looks
//! up bindings, supporting both single-stroke and chord sequences.

use std::collections::HashMap;

// ── Key types ────────────────────────────

/// A parsed key combination (one or more strokes).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeySpec {
    /// The strokes in this key combination.
    pub strokes: Vec<Stroke>,
}

/// A single keystroke with modifiers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Stroke {
    /// Ctrl modifier.
    pub ctrl: bool,
    /// Alt modifier.
    pub alt: bool,
    /// Shift modifier.
    pub shift: bool,
    /// The key itself.
    pub key: KeyName,
}

/// Named keys.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyName {
    /// A printable character.
    Char(char),
    /// Function key (1–12).
    F(u8),
    /// Arrow and navigation keys.
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    /// Special keys.
    Tab,
    Enter,
    Escape,
    Backspace,
    Delete,
    Space,
}

// ── Parsing ──────────────────────────────

/// Parse error for key specs.
#[derive(Debug, Clone)]
pub struct KeyParseError(pub String);

impl std::fmt::Display for KeyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for KeyParseError {}

impl KeySpec {
    /// Parse a key spec string like `"ctrl+s"` or `"ctrl+x ctrl+s"`.
    pub fn parse(s: &str) -> Result<Self, KeyParseError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(KeyParseError("empty key spec".into()));
        }
        let strokes: Result<Vec<Stroke>, KeyParseError> =
            s.split_whitespace().map(parse_stroke).collect();
        Ok(Self { strokes: strokes? })
    }
}

fn parse_stroke(s: &str) -> Result<Stroke, KeyParseError> {
    let parts: Vec<&str> = s.split('+').collect();
    if parts.is_empty() {
        return Err(KeyParseError("empty stroke".into()));
    }
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut key_part = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" => ctrl = true,
            "alt" => alt = true,
            "shift" => shift = true,
            _ => {
                if key_part.is_some() {
                    return Err(KeyParseError(format!("multiple keys in stroke: {s}")));
                }
                key_part = Some(*part);
            }
        }
    }

    let key_str = key_part.ok_or_else(|| KeyParseError(format!("no key in: {s}")))?;
    let key = parse_key_name(key_str)?;
    Ok(Stroke {
        ctrl,
        alt,
        shift,
        key,
    })
}

fn parse_key_name(s: &str) -> Result<KeyName, KeyParseError> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "up" => Ok(KeyName::Up),
        "down" => Ok(KeyName::Down),
        "left" => Ok(KeyName::Left),
        "right" => Ok(KeyName::Right),
        "home" => Ok(KeyName::Home),
        "end" => Ok(KeyName::End),
        "pageup" => Ok(KeyName::PageUp),
        "pagedown" => Ok(KeyName::PageDown),
        "tab" => Ok(KeyName::Tab),
        "enter" => Ok(KeyName::Enter),
        "escape" | "esc" => Ok(KeyName::Escape),
        "backspace" => Ok(KeyName::Backspace),
        "delete" | "del" => Ok(KeyName::Delete),
        "space" => Ok(KeyName::Space),
        _ if lower.starts_with('f') && lower.len() <= 3 => {
            let num: u8 = lower[1..]
                .parse()
                .map_err(|_| KeyParseError(format!("invalid key: {s}")))?;
            if (1..=12).contains(&num) {
                Ok(KeyName::F(num))
            } else {
                Err(KeyParseError(format!("invalid function key: {s}")))
            }
        }
        _ if s.len() == 1 => {
            let ch = s.chars().next().unwrap_or(' ');
            Ok(KeyName::Char(ch.to_ascii_lowercase()))
        }
        _ => Err(KeyParseError(format!("unknown key: {s}"))),
    }
}

// ── Binding table ────────────────────────

/// Where a binding was defined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingSource {
    /// Built-in default.
    Default,
    /// From `~/.kairnrc.tcl`.
    GlobalConfig,
    /// From `$PWD/.kairnrc.tcl`.
    ProjectConfig,
    /// Set at runtime via scripting.
    Runtime,
}

/// A bound action: script + source.
#[derive(Clone, Debug)]
pub struct BoundAction {
    /// The rusticle script to execute.
    pub script: String,
    /// Where this binding was defined.
    pub source: BindingSource,
}

/// Stores keybindings and resolves key events to scripts.
pub struct BindingTable {
    singles: HashMap<Stroke, BoundAction>,
    chords: HashMap<Stroke, HashMap<Stroke, BoundAction>>,
}

impl BindingTable {
    /// Create an empty binding table.
    pub fn new() -> Self {
        Self {
            singles: HashMap::new(),
            chords: HashMap::new(),
        }
    }

    /// Register a binding.
    pub fn bind(&mut self, spec: KeySpec, action: BoundAction) {
        match spec.strokes.len() {
            1 => {
                let mut iter = spec.strokes.into_iter();
                if let Some(stroke) = iter.next() {
                    if action.script.is_empty() {
                        self.singles.remove(&stroke);
                    } else {
                        self.singles.insert(stroke, action);
                    }
                }
            }
            2 => {
                let mut iter = spec.strokes.into_iter();
                if let (Some(first), Some(second)) = (iter.next(), iter.next()) {
                    let map = self.chords.entry(first).or_default();
                    if action.script.is_empty() {
                        map.remove(&second);
                    } else {
                        map.insert(second, action);
                    }
                }
            }
            _ => {} // ignore invalid lengths
        }
    }

    /// Look up a single-stroke binding.
    pub fn lookup_single(&self, stroke: &Stroke) -> Option<&BoundAction> {
        self.singles.get(stroke)
    }

    /// Check if a stroke is a chord prefix.
    pub fn is_chord_prefix(&self, stroke: &Stroke) -> bool {
        self.chords.get(stroke).is_some_and(|m| !m.is_empty())
    }

    /// Look up a two-stroke chord binding.
    pub fn lookup_chord(&self, first: &Stroke, second: &Stroke) -> Option<&BoundAction> {
        self.chords.get(first)?.get(second)
    }

    /// Iterate all bindings (for help display).
    pub fn all_bindings(&self) -> Vec<(String, &BoundAction)> {
        let mut out = Vec::new();
        for (stroke, action) in &self.singles {
            out.push((format_stroke(stroke), action));
        }
        for (first, seconds) in &self.chords {
            for (second, action) in seconds {
                out.push((
                    format!("{} {}", format_stroke(first), format_stroke(second)),
                    action,
                ));
            }
        }
        out
    }
}

impl Default for BindingTable {
    fn default() -> Self {
        Self::new()
    }
}

fn format_stroke(s: &Stroke) -> String {
    let mut parts = Vec::new();
    if s.ctrl {
        parts.push("ctrl");
    }
    if s.alt {
        parts.push("alt");
    }
    if s.shift {
        parts.push("shift");
    }
    parts.push(match &s.key {
        KeyName::Char(c) => return format_with_parts(&parts, &c.to_string()),
        KeyName::F(n) => return format_with_parts(&parts, &format!("f{n}")),
        KeyName::Up => "up",
        KeyName::Down => "down",
        KeyName::Left => "left",
        KeyName::Right => "right",
        KeyName::Home => "home",
        KeyName::End => "end",
        KeyName::PageUp => "pageup",
        KeyName::PageDown => "pagedown",
        KeyName::Tab => "tab",
        KeyName::Enter => "enter",
        KeyName::Escape => "escape",
        KeyName::Backspace => "backspace",
        KeyName::Delete => "delete",
        KeyName::Space => "space",
    });
    parts.join("+")
}

fn format_with_parts(parts: &[&str], key: &str) -> String {
    if parts.is_empty() {
        key.to_string()
    } else {
        format!("{}+{key}", parts.join("+"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_key() {
        let ks = KeySpec::parse("ctrl+s").unwrap();
        assert_eq!(ks.strokes.len(), 1);
        assert!(ks.strokes[0].ctrl);
        assert!(!ks.strokes[0].alt);
        assert_eq!(ks.strokes[0].key, KeyName::Char('s'));
    }

    #[test]
    fn parse_chord() {
        let ks = KeySpec::parse("ctrl+x ctrl+s").unwrap();
        assert_eq!(ks.strokes.len(), 2);
        assert!(ks.strokes[0].ctrl);
        assert_eq!(ks.strokes[0].key, KeyName::Char('x'));
        assert!(ks.strokes[1].ctrl);
        assert_eq!(ks.strokes[1].key, KeyName::Char('s'));
    }

    #[test]
    fn parse_function_key() {
        let ks = KeySpec::parse("f12").unwrap();
        assert_eq!(ks.strokes[0].key, KeyName::F(12));
    }

    #[test]
    fn parse_shift_modifier() {
        let ks = KeySpec::parse("ctrl+shift+up").unwrap();
        assert!(ks.strokes[0].ctrl);
        assert!(ks.strokes[0].shift);
        assert_eq!(ks.strokes[0].key, KeyName::Up);
    }

    #[test]
    fn parse_alt_modifier() {
        let ks = KeySpec::parse("alt+left").unwrap();
        assert!(ks.strokes[0].alt);
        assert_eq!(ks.strokes[0].key, KeyName::Left);
    }

    #[test]
    fn parse_invalid_returns_error() {
        assert!(KeySpec::parse("").is_err());
        assert!(KeySpec::parse("ctrl+").is_err());
        assert!(KeySpec::parse("bogus+x").is_err());
    }

    #[test]
    fn bind_and_lookup_single() {
        let mut table = BindingTable::new();
        let ks = KeySpec::parse("ctrl+s").unwrap();
        table.bind(
            ks.clone(),
            BoundAction {
                script: "buffer save".into(),
                source: BindingSource::Default,
            },
        );
        let action = table.lookup_single(&ks.strokes[0]);
        assert!(action.is_some());
        assert_eq!(action.unwrap().script, "buffer save");
    }

    #[test]
    fn later_bind_replaces_earlier() {
        let mut table = BindingTable::new();
        let ks = KeySpec::parse("ctrl+s").unwrap();
        table.bind(
            ks.clone(),
            BoundAction {
                script: "buffer save".into(),
                source: BindingSource::Default,
            },
        );
        table.bind(
            ks.clone(),
            BoundAction {
                script: "buffer save-all".into(),
                source: BindingSource::GlobalConfig,
            },
        );
        let action = table.lookup_single(&ks.strokes[0]).unwrap();
        assert_eq!(action.script, "buffer save-all");
    }

    #[test]
    fn empty_script_unbinds() {
        let mut table = BindingTable::new();
        let ks = KeySpec::parse("ctrl+q").unwrap();
        table.bind(
            ks.clone(),
            BoundAction {
                script: "editor quit".into(),
                source: BindingSource::Default,
            },
        );
        table.bind(
            ks.clone(),
            BoundAction {
                script: String::new(),
                source: BindingSource::GlobalConfig,
            },
        );
        assert!(table.lookup_single(&ks.strokes[0]).is_none());
    }

    #[test]
    fn chord_lookup() {
        let mut table = BindingTable::new();
        let ks = KeySpec::parse("ctrl+x ctrl+s").unwrap();
        table.bind(
            ks.clone(),
            BoundAction {
                script: "buffer save".into(),
                source: BindingSource::Default,
            },
        );
        assert!(table.is_chord_prefix(&ks.strokes[0]));
        let action = table.lookup_chord(&ks.strokes[0], &ks.strokes[1]);
        assert!(action.is_some());
        assert_eq!(action.unwrap().script, "buffer save");
    }

    #[test]
    fn all_bindings_lists_everything() {
        let mut table = BindingTable::new();
        table.bind(
            KeySpec::parse("ctrl+s").unwrap(),
            BoundAction {
                script: "save".into(),
                source: BindingSource::Default,
            },
        );
        table.bind(
            KeySpec::parse("ctrl+x ctrl+s").unwrap(),
            BoundAction {
                script: "save-all".into(),
                source: BindingSource::Default,
            },
        );
        let all = table.all_bindings();
        assert_eq!(all.len(), 2);
    }
}
