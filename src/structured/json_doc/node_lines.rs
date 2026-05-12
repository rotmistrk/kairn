//! Maps nodes to their line positions in stripped JSON source.
//! Used by JSONC parser to attach comments to the correct nodes.

use crate::structured::NodeId;

use super::JsonDoc;

/// Map nodes to their line numbers in the stripped JSON source.
/// Walks the stripped JSON and assigns line numbers to nodes in DFS order.
pub(crate) fn compute_node_lines(stripped: &str, doc: &JsonDoc) -> Vec<(NodeId, usize)> {
    let mut result = Vec::new();
    let mut node_idx = 0;
    let mut line = 0usize;
    let chars: Vec<char> = stripped.chars().collect();
    let mut i = 0;

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
        match ch {
            '{' | '[' => {
                result.push((NodeId(node_idx), line));
                node_idx += 1;
                i += 1;
            }
            '"' => {
                i += 1;
                // Read to end of string
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
                // Check if this was a key (followed by ':') or a value
                let after = skip_ws(&chars, i);
                if after < chars.len() && chars[after] == ':' {
                    // Key — don't count as a node, advance past ':'
                    i = after + 1;
                } else {
                    // String value
                    result.push((NodeId(node_idx), line));
                    node_idx += 1;
                }
            }
            't' | 'f' | 'n' => {
                result.push((NodeId(node_idx), line));
                node_idx += 1;
                while i < chars.len() && chars[i].is_alphabetic() {
                    i += 1;
                }
            }
            '-' | '0'..='9' => {
                result.push((NodeId(node_idx), line));
                node_idx += 1;
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
            }
            _ => {
                i += 1;
            }
        }
    }
    result
}

fn skip_ws(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t' || chars[i] == '\r' || chars[i] == '\n') {
        i += 1;
    }
    i
}
