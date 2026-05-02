//! Core parser types and top-level parsing logic.

use crate::error::TclError;

/// A parsed command: name + arguments, all as raw strings.
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    /// The words of the command (first word is the command name).
    pub words: Vec<Word>,
    /// Line number where this command starts (1-based).
    pub line: usize,
}

/// A single word in a command, preserving how it was written.
#[derive(Clone, Debug, PartialEq)]
pub enum Word {
    /// A literal string (no substitution needed).
    Literal(String),
    /// A braced string {…} (no substitution).
    Braced(String),
    /// A quoted string "…" (needs variable/command substitution).
    Quoted(String),
    /// A bare word (needs variable/command substitution).
    Bare(String),
    /// A structured dict literal %{…}.
    DictLiteral(String),
    /// A structured list literal %[…].
    ListLiteral(String),
    /// A heredoc with substitution <<TAG…TAG.
    Heredoc(String),
    /// A heredoc without substitution <<'TAG'…TAG.
    HeredocRaw(String),
}

impl Word {
    /// Get the raw text content of this word.
    pub fn text(&self) -> &str {
        match self {
            Self::Literal(s)
            | Self::Braced(s)
            | Self::Quoted(s)
            | Self::Bare(s)
            | Self::DictLiteral(s)
            | Self::ListLiteral(s)
            | Self::Heredoc(s)
            | Self::HeredocRaw(s) => s,
        }
    }
}

/// A parsed script: a sequence of commands.
#[derive(Clone, Debug)]
pub struct ParsedScript {
    /// The commands in this script.
    pub commands: Vec<Command>,
}

/// The parser.
pub struct Parser;

impl Parser {
    /// Parse a script into a sequence of commands.
    pub fn parse(input: &str) -> Result<ParsedScript, TclError> {
        let preprocessed = super::subst::preprocess_pipes(input);
        let mut commands = Vec::new();
        let chars: Vec<char> = preprocessed.chars().collect();
        let mut pos = 0;
        let mut line = 1;

        while pos < chars.len() {
            skip_whitespace_and_comments(&chars, &mut pos, &mut line);
            if pos >= chars.len() {
                break;
            }
            let cmd_line = line;
            let words = super::words::parse_command_words(&chars, &mut pos, &mut line)?;
            if !words.is_empty() {
                commands.push(Command {
                    words,
                    line: cmd_line,
                });
            }
        }

        Ok(ParsedScript { commands })
    }
}

/// Skip whitespace, newlines, comments, and semicolons.
fn skip_whitespace_and_comments(chars: &[char], pos: &mut usize, line: &mut usize) {
    while *pos < chars.len() {
        let ch = chars[*pos];
        if ch == '\n' {
            *pos += 1;
            *line += 1;
        } else if ch == '\r' {
            *pos += 1;
            if *pos < chars.len() && chars[*pos] == '\n' {
                *pos += 1;
            }
            *line += 1;
        } else if ch == ' ' || ch == '\t' || ch == ';' {
            *pos += 1;
        } else if ch == '#' {
            // Skip to end of line
            while *pos < chars.len() && chars[*pos] != '\n' {
                *pos += 1;
            }
        } else {
            break;
        }
    }
}
