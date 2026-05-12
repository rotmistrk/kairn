//! Grep — pure Rust async project search. No external tools.
//! Uses `ignore` crate (respects .gitignore) + `regex` for matching.
//! Supports POSIX-style flags: -i (case-insensitive), -E (extended/default),
//! -F (fixed string), -w (word boundary), -l (files only), -n (line numbers, default).
//! Quoting: "Few Words" or 'single quotes' for patterns with spaces.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;

use ignore::WalkBuilder;
use regex::RegexBuilder;
use txv_core::run::Waker;

use crate::task_output::TaskOutput;
use crate::views::results::ResultEntry;

/// Parsed grep options.
struct GrepOpts {
    pattern: String,
    case_insensitive: bool,
    fixed_string: bool,
    word_boundary: bool,
}

/// Parse a grep command line (POSIX-style flags + pattern).
/// Supports: -i, -E, -F, -w, and quoted patterns.
fn parse_grep_args(input: &str) -> Result<GrepOpts, String> {
    let words = shell_words::split(input).map_err(|e| format!("Bad quoting: {e}"))?;
    let mut case_insensitive = false;
    let mut fixed_string = false;
    let mut word_boundary = false;
    let mut pattern_parts: Vec<String> = Vec::new();
    let mut flags_done = false;

    for word in &words {
        if !flags_done && word.starts_with('-') && word.len() > 1 && !word.starts_with("--") {
            for ch in word[1..].chars() {
                match ch {
                    'i' => case_insensitive = true,
                    'E' => {} // extended regex is default
                    'F' => fixed_string = true,
                    'w' => word_boundary = true,
                    'n' | 'l' | 'H' => {} // accepted but no-op (always show lines+files)
                    _ => return Err(format!("Unknown flag: -{ch}")),
                }
            }
        } else {
            flags_done = true;
            pattern_parts.push(word.clone());
        }
    }

    if pattern_parts.is_empty() {
        return Err("No pattern specified".to_string());
    }

    let pattern = pattern_parts.join(" ");
    Ok(GrepOpts {
        pattern,
        case_insensitive,
        fixed_string,
        word_boundary,
    })
}

/// Build a regex from parsed options.
fn build_regex(opts: &GrepOpts) -> Result<regex::Regex, String> {
    let mut pat = if opts.fixed_string {
        regex::escape(&opts.pattern)
    } else {
        opts.pattern.clone()
    };

    if opts.word_boundary {
        pat = format!(r"\b{pat}\b");
    }

    RegexBuilder::new(&pat)
        .case_insensitive(opts.case_insensitive)
        .build()
        .map_err(|e| format!("Invalid regex: {e}"))
}

/// GrepState is now TaskOutput (shared with build).
pub type GrepState = TaskOutput;

/// Spawn async grep. Parses POSIX flags from the input string.
/// Example inputs: `-i "hello world"`, `-iF fixed`, `-w MyStruct`
pub fn grep_async(input: &str, root: &Path, waker: Waker) -> Arc<GrepState> {
    let state = TaskOutput::new();
    let state_clone = state.clone();
    let input = input.to_string();
    let root = root.to_path_buf();

    std::thread::spawn(move || {
        let opts = match parse_grep_args(&input) {
            Ok(o) => o,
            Err(e) => {
                state_clone.set_error(e);
                state_clone.mark_done();
                waker.wake();
                return;
            }
        };

        let re = match build_regex(&opts) {
            Ok(r) => r,
            Err(e) => {
                state_clone.set_error(e);
                state_clone.mark_done();
                waker.wake();
                return;
            }
        };

        let walker = WalkBuilder::new(&root)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .build();

        let mut count = 0;
        let mut batch = Vec::with_capacity(16);

        for entry in walker.flatten() {
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            let file = match File::open(path) {
                Ok(f) => f,
                Err(_) => continue,
            };
            let reader = BufReader::new(file);
            let mut file_matches = 0;

            for (line_idx, line) in reader.lines().enumerate() {
                let Ok(line) = line else {
                    break;
                };
                if re.is_match(&line) {
                    batch.push(ResultEntry {
                        path: path.to_path_buf(),
                        line: line_idx as u32,
                        col: 0,
                        text: line.chars().take(200).collect(),
                    });
                    count += 1;
                    file_matches += 1;
                    if file_matches >= 10 {
                        break;
                    }
                    if batch.len() >= 16 {
                        state_clone.push_entries(&mut batch);
                        waker.wake();
                    }
                }
            }
            if count >= 1000 {
                break;
            }
        }

        if !batch.is_empty() {
            state_clone.push_entries(&mut batch);
        }
        state_clone.mark_done();
        waker.wake();
    });

    state
}

#[cfg(test)]
#[path = "grep_tests.rs"]
mod tests;
