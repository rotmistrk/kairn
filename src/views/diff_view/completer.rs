//! Completer for DiffView command line — suggests diff options and refs.

use txv_core::complete::{Completer, CompletionVisitor};

use crate::completer_entry::Entry;

const DIFF_OPTIONS: &[&str] = &["-y", "-w", "-U3", "-C"];
const DIFF_REFS: &[&str] = &["HEAD", "HEAD~1", "HEAD~2", "HEAD~3"];
const DIFF_COMMANDS: &[&str] = &["diff", "vdiff", "revert", "nodiff", "q"];

/// Completer for DiffView's command line.
pub(crate) struct DiffCompleter;

impl Completer for DiffCompleter {
    fn complete(
        &self,
        input: &str,
        _cursor: usize,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let last_word = input.trim().rsplit(' ').next().unwrap_or("");
        let candidates = if last_word.starts_with('-') {
            DIFF_OPTIONS
        } else if last_word.starts_with("HEAD") {
            DIFF_REFS
        } else {
            DIFF_COMMANDS
        };
        for &c in candidates {
            if c.starts_with(last_word) && c != last_word {
                let e = Entry {
                    text: c.to_string(),
                    display: c.to_string(),
                    kind: "diff",
                };
                if !visitor(&e)? {
                    break;
                }
            }
        }
        Ok(())
    }
}
