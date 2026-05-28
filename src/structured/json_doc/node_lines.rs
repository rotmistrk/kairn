//! Maps nodes to their line positions in stripped JSON source.
//! Used by JSONC parser to attach comments to the correct nodes.

use crate::structured::NodeId;

use super::JsonDoc;

/// Map nodes to their line numbers in the stripped JSON source.
/// Walks the stripped JSON and assigns line numbers to nodes in DFS order.
pub(crate) fn compute_node_lines(stripped: &str, doc: &JsonDoc) -> Vec<(NodeId, usize)> {
    let mut result = Vec::new();
    let mut node_idx = 0;
    let chars: Vec<char> = stripped.chars().collect();
    let mut i = 0;
    let mut line = 0usize;

    while i < chars.len() && node_idx < doc.nodes.len() {
        let ch = chars[i];
        if ch == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if ch.is_whitespace() || ch == ',' || ch == ':' || ch == '}' || ch == ']' {
            i += 1;
            continue;
        }
        i = consume_token(&chars, i, ch, &mut result, &mut node_idx, line);
    }
    result
}

fn consume_token(
    chars: &[char],
    i: usize,
    ch: char,
    result: &mut Vec<(NodeId, usize)>,
    node_idx: &mut usize,
    line: usize,
) -> usize {
    match ch {
        '{' | '[' => {
            result.push((NodeId(*node_idx), line));
            *node_idx += 1;
            i + 1
        }
        '"' => scan_string_token(chars, i, result, node_idx, line),
        't' | 'f' | 'n' => {
            result.push((NodeId(*node_idx), line));
            *node_idx += 1;
            skip_alpha(chars, i)
        }
        '-' | '0'..='9' => {
            result.push((NodeId(*node_idx), line));
            *node_idx += 1;
            skip_number(chars, i)
        }
        _ => i + 1,
    }
}

fn scan_string_token(
    chars: &[char],
    start: usize,
    result: &mut Vec<(NodeId, usize)>,
    node_idx: &mut usize,
    line: usize,
) -> usize {
    let mut i = start + 1;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
            continue;
        }
        if chars[i] == '"' {
            i += 1;
            break;
        }
        i += 1;
    }
    let after = skip_ws(chars, i);
    if after < chars.len() && chars[after] == ':' {
        // Key — don't count as a node
        after + 1
    } else {
        result.push((NodeId(*node_idx), line));
        *node_idx += 1;
        i
    }
}

fn skip_alpha(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && chars[i].is_alphabetic() {
        i += 1;
    }
    i
}

fn skip_number(chars: &[char], mut i: usize) -> usize {
    while i < chars.len()
        && (chars[i].is_ascii_digit()
            || chars[i] == '.'
            || chars[i] == 'e'
            || chars[i] == 'E'
            || chars[i] == '+'
            || chars[i] == '-')
    {
        i += 1;
    }
    i
}

fn skip_ws(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\r' || chars[i] == '\n') {
        i += 1;
    }
    i
}
