//! Git gutter signs — per-line change indicators vs HEAD.

use std::path::Path;

use crate::diff::git_file_content;

/// Per-line gutter sign.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GutterSign {
    Added,
    Modified,
    Deleted, // shown on the line AFTER the deletion
}

/// Compute gutter signs by diffing buffer content against git HEAD.
pub fn compute_gutter_signs(root: &Path, rel_path: &str, current: &str) -> Vec<(usize, GutterSign)> {
    let base = match git_file_content(root, rel_path, "HEAD") {
        Ok(content) => content,
        Err(_) => return Vec::new(), // new file or not in git
    };
    diff_to_signs(&base, current)
}

fn diff_to_signs(old: &str, new: &str) -> Vec<(usize, GutterSign)> {
    use similar::{ChangeTag, TextDiff};
    let diff = TextDiff::from_lines(old, new);
    let mut signs = Vec::new();
    let mut new_line: usize = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => new_line += 1,
            ChangeTag::Insert => {
                if try_upgrade_to_modified(&mut signs, new_line) {
                    new_line += 1;
                    continue;
                }
                signs.push((new_line, GutterSign::Added));
                new_line += 1;
            }
            ChangeTag::Delete => signs.push((new_line, GutterSign::Deleted)),
        }
    }
    signs
}

/// If the previous sign at this line was Deleted, upgrade it to Modified.
fn try_upgrade_to_modified(signs: &mut [(usize, GutterSign)], line: usize) -> bool {
    if let Some((last_line, last_sign)) = signs.last_mut() {
        if *last_line == line && *last_sign == GutterSign::Deleted {
            *last_sign = GutterSign::Modified;
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_changes_no_signs() {
        let signs = diff_to_signs("a\nb\nc\n", "a\nb\nc\n");
        assert!(signs.is_empty());
    }

    #[test]
    fn added_line() {
        let signs = diff_to_signs("a\nc\n", "a\nb\nc\n");
        assert_eq!(signs, vec![(1, GutterSign::Added)]);
    }

    #[test]
    fn deleted_line() {
        let signs = diff_to_signs("a\nb\nc\n", "a\nc\n");
        assert_eq!(signs, vec![(1, GutterSign::Deleted)]);
    }

    #[test]
    fn modified_line() {
        let signs = diff_to_signs("a\nb\nc\n", "a\nB\nc\n");
        assert_eq!(signs, vec![(1, GutterSign::Modified)]);
    }

    #[test]
    fn multiple_changes() {
        let signs = diff_to_signs("a\nb\nc\nd\n", "a\nB\nc\nd\ne\n");
        assert_eq!(signs, vec![(1, GutterSign::Modified), (4, GutterSign::Added)]);
    }
}
