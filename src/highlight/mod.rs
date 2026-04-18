// Syntax highlighting wrapper around `syntect`.

use std::path::Path;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

/// Holds loaded syntax definitions and theme.
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme_name: "base16-eighties.dark".to_string(),
        }
    }

    /// Highlight source code, returning styled ratatui Lines.
    /// Falls back to plain text if syntax is unknown.
    pub fn highlight_file<'a>(&self, content: &'a str, file_path: &str) -> Vec<Line<'a>> {
        let syntax = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.syntax_set.find_syntax_by_extension(ext))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes[&self.theme_name];
        let mut h = HighlightLines::new(syntax, theme);

        LinesWithEndings::from(content)
            .map(|line| self.highlight_line(&mut h, line))
            .collect()
    }

    fn highlight_line<'a>(&self, h: &mut HighlightLines<'_>, line: &'a str) -> Line<'a> {
        let regions = h.highlight_line(line, &self.syntax_set).unwrap_or_default();

        let spans: Vec<Span<'a>> = regions
            .into_iter()
            .map(|(style, text)| {
                let fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                let mut ratatui_style = Style::default().fg(fg);
                if style.font_style.contains(FontStyle::BOLD) {
                    ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
                }
                if style.font_style.contains(FontStyle::ITALIC) {
                    ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
                }
                Span::styled(text, ratatui_style)
            })
            .collect();

        Line::from(spans)
    }
}
