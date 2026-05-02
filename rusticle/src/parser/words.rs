//! Word-level parsing: splitting commands into words.

use super::core::Word;
use crate::error::TclError;

/// Parse the words of a single command until end-of-line, semicolon, or EOF.
pub fn parse_command_words(
    chars: &[char],
    pos: &mut usize,
    line: &mut usize,
) -> Result<Vec<Word>, TclError> {
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

/// Check if position starts a heredoc `<<`.
fn peek_heredoc(chars: &[char], pos: usize) -> bool {
    pos + 1 < chars.len() && chars[pos + 1] == '<'
}

/// Parse a heredoc `<<TAG...TAG` or `<<'TAG'...TAG`.
fn parse_heredoc(chars: &[char], pos: &mut usize, line: &mut usize) -> Result<Word, TclError> {
    *pos += 2; // skip <<
    let raw = chars.get(*pos) == Some(&'\'');
    if raw {
        *pos += 1; // skip opening '
    }
    // Read tag name
    let mut tag = String::new();
    while *pos < chars.len() && chars[*pos] != '\n' && chars[*pos] != '\'' {
        tag.push(chars[*pos]);
        *pos += 1;
    }
    let tag = tag.trim().to_string();
    if raw && *pos < chars.len() && chars[*pos] == '\'' {
        *pos += 1; // skip closing '
    }
    // Skip to next line
    if *pos < chars.len() && chars[*pos] == '\n' {
        *pos += 1;
        *line += 1;
    }
    // Read content until we find the tag on its own line
    let mut content = String::new();
    let mut lines_buf: Vec<String> = Vec::new();
    let closing_indent = loop {
        if *pos >= chars.len() {
            return Err(TclError::new(format!(
                "unterminated heredoc, expected '{tag}'"
            )));
        }
        let line_start = *pos;
        while *pos < chars.len() && chars[*pos] != '\n' {
            *pos += 1;
        }
        let current_line: String = chars[line_start..*pos].iter().collect();
        if *pos < chars.len() {
            *pos += 1; // skip \n
            *line += 1;
        }
        if current_line.trim() == tag {
            break current_line.len() - current_line.trim_start().len();
        }
        lines_buf.push(current_line);
    };
    // Strip common indent based on closing tag position
    for (i, l) in lines_buf.iter().enumerate() {
        if i > 0 {
            content.push('\n');
        }
        if l.len() > closing_indent {
            content.push_str(&l[closing_indent..]);
        } else {
            content.push_str(l.trim_start());
        }
    }
    if raw {
        Ok(Word::HeredocRaw(content))
    } else {
        Ok(Word::Heredoc(content))
    }
}

/// Skip to end of line.
fn skip_to_eol(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos] != '\n' {
        *pos += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::super::core::Parser;
    use super::*;

    #[test]
    fn simple_command() {
        let script = Parser::parse("puts hello").unwrap();
        assert_eq!(script.commands.len(), 1);
        assert_eq!(script.commands[0].words.len(), 2);
        assert_eq!(script.commands[0].words[0].text(), "puts");
        assert_eq!(script.commands[0].words[1].text(), "hello");
    }

    #[test]
    fn braced_word() {
        let script = Parser::parse("set x {hello world}").unwrap();
        assert_eq!(script.commands[0].words.len(), 3);
        assert_eq!(script.commands[0].words[2].text(), "hello world");
    }

    #[test]
    fn quoted_word() {
        let script = Parser::parse(r#"puts "hello world""#).unwrap();
        assert_eq!(script.commands[0].words.len(), 2);
        assert_eq!(script.commands[0].words[1].text(), "hello world");
    }

    #[test]
    fn command_substitution() {
        let script = Parser::parse("puts [expr 1]").unwrap();
        assert_eq!(script.commands[0].words[1].text(), "[expr 1]");
    }

    #[test]
    fn multiple_commands() {
        let script = Parser::parse("set x 1\nset y 2").unwrap();
        assert_eq!(script.commands.len(), 2);
    }

    #[test]
    fn semicolon_separator() {
        let script = Parser::parse("set x 1; set y 2").unwrap();
        assert_eq!(script.commands.len(), 2);
    }

    #[test]
    fn comment_skipped() {
        let script = Parser::parse("# this is a comment\nputs hello").unwrap();
        assert_eq!(script.commands.len(), 1);
        assert_eq!(script.commands[0].words[0].text(), "puts");
    }

    #[test]
    fn dict_literal() {
        let script = Parser::parse(r#"set d %{ name: "kairn" }"#).unwrap();
        assert_eq!(script.commands[0].words.len(), 3);
        matches!(&script.commands[0].words[2], Word::DictLiteral(_));
    }

    #[test]
    fn list_literal() {
        let script = Parser::parse(r#"set l %[ "a", "b" ]"#).unwrap();
        assert_eq!(script.commands[0].words.len(), 3);
        matches!(&script.commands[0].words[2], Word::ListLiteral(_));
    }

    #[test]
    fn pipe_rewrite() {
        let script = Parser::parse("a | b | c").unwrap();
        // After pipe rewrite: "c [b [a]]"
        assert_eq!(script.commands.len(), 1);
        assert_eq!(script.commands[0].words[0].text(), "c");
    }

    #[test]
    fn nested_braces() {
        let script = Parser::parse("set x {a {b c} d}").unwrap();
        assert_eq!(script.commands[0].words[2].text(), "a {b c} d");
    }

    #[test]
    fn empty_braces() {
        let script = Parser::parse("set x {}").unwrap();
        assert_eq!(script.commands[0].words[2].text(), "");
    }

    #[test]
    fn unmatched_brace_error() {
        let result = Parser::parse("set x {unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn unmatched_quote_error() {
        let result = Parser::parse(r#"set x "unclosed"#);
        assert!(result.is_err());
    }

    #[test]
    fn heredoc_basic() {
        let script = "set x <<END\nhello world\nEND";
        let parsed = Parser::parse(script).unwrap();
        assert_eq!(parsed.commands.len(), 1);
        let word = &parsed.commands[0].words[2];
        assert!(matches!(word, Word::Heredoc(_)));
        assert_eq!(word.text(), "hello world");
    }

    #[test]
    fn heredoc_raw() {
        let script = "set x <<'END'\nhello $world\nEND";
        let parsed = Parser::parse(script).unwrap();
        let word = &parsed.commands[0].words[2];
        assert!(matches!(word, Word::HeredocRaw(_)));
        assert!(word.text().contains("$world"));
    }

    #[test]
    fn empty_script() {
        let script = Parser::parse("").unwrap();
        assert!(script.commands.is_empty());
    }

    #[test]
    fn whitespace_only() {
        let script = Parser::parse("   \n  \n  ").unwrap();
        assert!(script.commands.is_empty());
    }

    #[test]
    fn line_numbers_tracked() {
        let script = Parser::parse("set x 1\n\nset y 2").unwrap();
        assert_eq!(script.commands[0].line, 1);
        assert_eq!(script.commands[1].line, 3);
    }
}
