//! JSONC parser — strips comments from JSON while tracking their positions.

/// Kind of comment found in JSONC input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentKind {
    Line,
    Block,
}

/// A comment extracted from JSONC input.
#[derive(Debug, Clone)]
pub struct Comment {
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) text: String,
    pub(crate) kind: CommentKind,
}

/// Strip comments from JSONC input, returning clean JSON and extracted comments.
/// Handles `//` and `/* */` comments, skipping comment-like sequences inside strings.
pub fn strip_comments(input: &str) -> (String, Vec<Comment>) {
    let mut out = String::with_capacity(input.len());
    let mut comments = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut line = 0usize;
    let mut col = 0usize;
    let mut in_string = false;

    while i < len {
        let ch = chars[i];

        if in_string {
            out.push(ch);
            if ch == '\\' && i + 1 < len {
                out.push(chars[i + 1]);
                col += 2;
                i += 2;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            advance_pos(ch, &mut line, &mut col);
            i += 1;
            continue;
        }

        if ch == '"' {
            in_string = true;
            out.push(ch);
            col += 1;
            i += 1;
            continue;
        }

        if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
            let start_col = col;
            let start_line = line;
            i += 2;
            col += 2;
            let mut text = String::new();
            while i < len && chars[i] != '\n' {
                text.push(chars[i]);
                i += 1;
                col += 1;
            }
            comments.push(Comment {
                line: start_line,
                col: start_col,
                text: text.trim().to_string(),
                kind: CommentKind::Line,
            });
            continue;
        }

        if ch == '/' && i + 1 < len && chars[i + 1] == '*' {
            let start_col = col;
            let start_line = line;
            i += 2;
            col += 2;
            let mut text = String::new();
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                text.push(chars[i]);
                advance_pos(chars[i], &mut line, &mut col);
                i += 1;
            }
            if i + 1 < len {
                i += 2;
                col += 2;
            }
            comments.push(Comment {
                line: start_line,
                col: start_col,
                text: text.trim().to_string(),
                kind: CommentKind::Block,
            });
            continue;
        }

        out.push(ch);
        advance_pos(ch, &mut line, &mut col);
        i += 1;
    }

    (out, comments)
}

fn advance_pos(ch: char, line: &mut usize, col: &mut usize) {
    if ch == '\n' {
        *line += 1;
        *col = 0;
    } else {
        *col += 1;
    }
}
