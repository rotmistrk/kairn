//! Word-level parsing: splitting commands into words.

use super::core::Word;
use super::words_ext::{parse_heredoc, peek_heredoc, skip_to_eol};
use crate::error::TclError;

/// Parse the words of a single command until end-of-line, semicolon, or EOF.
pub fn parse_command_words(chars: &[char], pos: &mut usize, line: &mut usize) -> Result<Vec<Word>, TclError> {
    let mut words = Vec::new();

    loop {
        // Skip horizontal whitespace
        while *pos < chars.len() && (chars[*pos] == ' ' || chars[*pos] == '\t') {
            *pos += 1;
        }
        if *pos >= chars.len() {
            break;
        }
        let ch = chars[*pos];
        // End of command
        if ch == '\n' || ch == '\r' || ch == ';' {
            break;
        }
        // Comment at start of command
        if ch == '#' && words.is_empty() {
            skip_to_eol(chars, pos);
            break;
        }
        let word = parse_one_word(chars, pos, line)?;
        let is_heredoc = matches!(&word, Word::Heredoc(_) | Word::HeredocRaw(_));
        words.push(word);
        // Heredoc consumes multiple lines; end the current command after it
        if is_heredoc {
            break;
        }
    }
    Ok(words)
}

/// Parse a single word.
fn parse_one_word(chars: &[char], pos: &mut usize, line: &mut usize) -> Result<Word, TclError> {
    let ch = chars[*pos];

    if ch == '{' {
        return parse_braced(chars, pos, line);
    }
    if ch == '"' {
        return parse_quoted(chars, pos, line);
    }
    if ch == '[' {
        return parse_bracket(chars, pos, line);
    }
    // Structured literals
    if ch == '%' && *pos + 1 < chars.len() {
        let next = chars[*pos + 1];
        if next == '{' {
            *pos += 2;
            let content = super::literals::parse_dict_literal(chars, pos, line)?;
            return Ok(Word::DictLiteral(content));
        }
        if next == '[' {
            *pos += 2;
            let content = super::literals::parse_list_literal(chars, pos, line)?;
            return Ok(Word::ListLiteral(content));
        }
    }
    // Heredoc
    if ch == '<' && peek_heredoc(chars, *pos) {
        return parse_heredoc(chars, pos, line);
    }
    parse_bare(chars, pos, line)
}

/// Parse a braced word `{...}`.
fn parse_braced(chars: &[char], pos: &mut usize, line: &mut usize) -> Result<Word, TclError> {
    *pos += 1; // skip {
    let mut depth = 1;
    let mut content = String::new();
    while *pos < chars.len() && depth > 0 {
        let ch = chars[*pos];
        if ch == '\n' {
            *line += 1;
        }
        if ch == '\\' && *pos + 1 < chars.len() && chars[*pos + 1] == '\n' {
            // Line continuation inside braces: keep literal
            content.push('\\');
            content.push('\n');
            *pos += 2;
            *line += 1;
            continue;
        }
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                *pos += 1;
                return Ok(Word::Braced(content));
            }
        }
        content.push(ch);
        *pos += 1;
    }
    Err(TclError::new("unmatched '{'"))
}

/// Parse a quoted word `"..."`.
fn parse_quoted(chars: &[char], pos: &mut usize, line: &mut usize) -> Result<Word, TclError> {
    *pos += 1; // skip "
    let mut content = String::new();
    while *pos < chars.len() && chars[*pos] != '"' {
        let ch = chars[*pos];
        if ch == '\n' {
            *line += 1;
        }
        if ch == '\\' && *pos + 1 < chars.len() {
            content.push('\\');
            *pos += 1;
            content.push(chars[*pos]);
            *pos += 1;
            continue;
        }
        content.push(ch);
        *pos += 1;
    }
    if *pos >= chars.len() {
        return Err(TclError::new("unmatched '\"'"));
    }
    *pos += 1; // skip closing "
    Ok(Word::Quoted(content))
}

/// Parse a bracket-enclosed command substitution `[...]`.
fn parse_bracket(chars: &[char], pos: &mut usize, line: &mut usize) -> Result<Word, TclError> {
    *pos += 1; // skip [
    let mut depth = 1;
    let mut content = String::new();
    while *pos < chars.len() && depth > 0 {
        let ch = chars[*pos];
        if ch == '\n' {
            *line += 1;
        }
        if ch == '[' {
            depth += 1;
        } else if ch == ']' {
            depth -= 1;
            if depth == 0 {
                *pos += 1;
                return Ok(Word::Bare(format!("[{content}]")));
            }
        }
        content.push(ch);
        *pos += 1;
    }
    Err(TclError::new("unmatched '['"))
}

/// Parse a bare (unquoted) word.
fn parse_bare(chars: &[char], pos: &mut usize, _line: &mut usize) -> Result<Word, TclError> {
    let mut content = String::new();
    while *pos < chars.len() {
        let ch = chars[*pos];
        if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' || ch == ';' {
            break;
        }
        if ch == '\\' && *pos + 1 < chars.len() {
            if chars[*pos + 1] == '\n' {
                // Line continuation: skip backslash-newline and following whitespace
                *pos += 2;
                while *pos < chars.len() && (chars[*pos] == ' ' || chars[*pos] == '\t') {
                    *pos += 1;
                }
                content.push(' ');
                continue;
            }
            content.push('\\');
            *pos += 1;
            content.push(chars[*pos]);
            *pos += 1;
            continue;
        }
        content.push(ch);
        *pos += 1;
    }
    Ok(Word::Bare(content))
}

