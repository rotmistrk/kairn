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
                    let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
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
