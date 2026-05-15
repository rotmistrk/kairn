//! Syntax highlighting via syntect — detects language from extension, highlights lines.

use std::path::Path;

use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

use txv_core::prelude::{Color, Style};

/// A span of highlighted text.
pub struct HlSpan {
    pub text: String,
    pub style: Style,
}

/// Caches syntax sets and per-file highlighter state.
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
    themes: ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::with_theme("base16-eighties.dark")
    }
}

impl Highlighter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a specific theme name. Falls back to first available if not found.
    pub fn with_theme(name: &str) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = ThemeSet::load_defaults();
        let theme = themes
            .themes
            .get(name)
            .cloned()
            .unwrap_or_else(|| themes.themes.values().next().unwrap().clone());
        Self {
            syntax_set,
            theme,
            themes,
        }
    }

    /// List available theme names.
    pub fn available_themes(&self) -> Vec<&str> {
        self.themes.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Access the syntax set.
    pub fn syntax_set(&self) -> &SyntaxSet {
        &self.syntax_set
    }

    /// Access the current theme.
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Switch to a different theme by name. Returns true if found.
    pub fn set_theme(&mut self, name: &str) -> bool {
        if let Some(t) = self.themes.themes.get(name) {
            self.theme = t.clone();
            true
        } else {
            false
        }
    }

    /// Highlight a single line of text for the given file extension.
    pub fn highlight_line(&self, line: &str, ext: &str) -> Vec<HlSpan> {
        let syntax = match self.syntax_set.find_syntax_by_extension(ext) {
            Some(s) => s,
            None => {
                return vec![HlSpan {
                    text: line.to_string(),
                    style: Style::default(),
                }]
            }
        };

        use syntect::easy::HighlightLines;
        let mut h = HighlightLines::new(syntax, &self.theme);
        match h.highlight_line(line, &self.syntax_set) {
            Ok(ranges) => ranges
                .iter()
                .map(|(style, text)| {
                    let (r, g, b) = ensure_readable(style.foreground.r, style.foreground.g, style.foreground.b);
                    let fg = Color::Rgb(r, g, b);
                    HlSpan {
                        text: text.to_string(),
                        style: Style { fg, ..Style::default() },
                    }
                })
                .collect(),
            Err(_) => vec![HlSpan {
                text: line.to_string(),
                style: Style::default(),
            }],
        }
    }
}

/// Extract file extension from a path.
pub fn extension_from_path(path: &Path) -> &str {
    path.extension().and_then(|e| e.to_str()).unwrap_or("")
}

/// Ensure foreground color is readable on a dark background.
/// If brightness is below threshold, clamp to a minimum visible gray.
pub(crate) fn ensure_readable(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let brightness = (r as u16 + g as u16 + b as u16) / 3;
    if brightness < 80 {
        let floor: u8 = 120;
        (r.max(floor), g.max(floor), b.max(floor))
    } else {
        (r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_brackets_readable_after_fix() {
        let hl = Highlighter::with_theme("base16-eighties.dark");
        let spans = hl.highlight_line("}", "java");
        for span in &spans {
            if span.text.contains('}') {
                let Color::Rgb(r, g, b) = span.style.fg else {
                    panic!("not rgb")
                };
                eprintln!("  java bracket -> ({r},{g},{b})");
                assert!(r >= 120 || g >= 120 || b >= 120, "too dark: ({r},{g},{b})");
            }
        }
    }

    #[test]
    fn ensure_readable_clamps_dark() {
        assert_eq!(ensure_readable(45, 45, 45), (120, 120, 120));
        assert_eq!(ensure_readable(200, 200, 200), (200, 200, 200));
        assert_eq!(ensure_readable(0, 0, 100), (120, 120, 120));
    }
}
