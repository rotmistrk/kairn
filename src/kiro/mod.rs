//! Kiro AI integration: send selections/files to Kiro, detect and apply diffs.

use std::fmt;

/// Text payload to send to the Kiro terminal tab.
#[derive(Debug, Clone)]
pub struct KiroPayload {
    /// The text to inject into the Kiro PTY.
    pub text: String,
    /// Optional user prompt to prepend.
    pub prompt: Option<String>,
}

impl KiroPayload {
    /// Build a payload from a file path and optional line range.
    pub fn from_file(path: &str, start: usize, end: usize) -> Self {
        let text = format!("@file {path}:{start}-{end}");
        Self { text, prompt: None }
    }

    /// Build a payload from selected text.
    pub fn from_selection(selection: &str) -> Self {
        Self {
            text: selection.to_string(),
            prompt: None,
        }
    }

    /// Attach a user prompt.
    pub fn with_prompt(mut self, prompt: &str) -> Self {
        self.prompt = Some(prompt.to_string());
        self
    }

    /// Format the final string to send to the Kiro PTY.
    pub fn to_pty_input(&self) -> String {
        match &self.prompt {
            Some(p) => format!("{}\n{}\n", p, self.text),
            None => format!("{}\n", self.text),
        }
    }
}

/// A code block detected in terminal output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    /// Language tag from the opening fence (e.g. "rust", "go").
    pub language: String,
    /// The code content (without fences).
    pub content: String,
    /// Starting line index in the terminal buffer.
    pub start_line: usize,
    /// Ending line index in the terminal buffer.
    pub end_line: usize,
}

impl fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "```{}\n{}\n```", self.language, self.content)
    }
}

/// Scan terminal output lines for fenced code blocks (triple-backtick).
pub fn detect_code_blocks(lines: &[&str]) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut language = String::new();
    let mut content = String::new();
    let mut start_line = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !in_block && trimmed.starts_with("```") {
            in_block = true;
            language = trimmed[3..].trim().to_string();
            content.clear();
            start_line = i;
        } else if in_block && trimmed == "```" {
            blocks.push(CodeBlock {
                language: language.clone(),
                content: content.trim_end().to_string(),
                start_line,
                end_line: i,
            });
            in_block = false;
        } else if in_block {
            if !content.is_empty() {
                content.push('\n');
            }
            content.push_str(line);
        }
    }
    blocks
}

/// Compute a simple line-based diff between old and new content.
/// Returns lines formatted as a unified-style preview.
pub fn compute_diff_preview(old: &str, new: &str) -> Vec<DiffLine> {
    use similar::ChangeTag;
    let diff = similar::TextDiff::from_lines(old, new);
    let mut result = Vec::new();
    for change in diff.iter_all_changes() {
        let tag = match change.tag() {
            ChangeTag::Delete => DiffTag::Remove,
            ChangeTag::Insert => DiffTag::Add,
            ChangeTag::Equal => DiffTag::Context,
        };
        result.push(DiffLine {
            tag,
            text: change.value().to_string(),
        });
    }
    result
}

/// A single line in a diff preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub text: String,
}

/// Tag for diff lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    Add,
    Remove,
    Context,
}

impl DiffTag {
    /// Prefix character for display.
    pub fn prefix(self) -> char {
        match self {
            Self::Add => '+',
            Self::Remove => '-',
            Self::Context => ' ',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_from_file() {
        let p = KiroPayload::from_file("src/main.rs", 10, 20);
        assert_eq!(p.to_pty_input(), "@file src/main.rs:10-20\n");
    }

    #[test]
    fn payload_with_prompt() {
        let p = KiroPayload::from_selection("fn main() {}").with_prompt("explain this");
        assert_eq!(p.to_pty_input(), "explain this\nfn main() {}\n");
    }

    #[test]
    fn detect_single_code_block() {
        let lines = vec![
            "Here is the fix:",
            "```rust",
            "fn hello() {",
            "    println!(\"hi\");",
            "}",
            "```",
            "Done.",
        ];
        let blocks = detect_code_blocks(&lines);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].language, "rust");
        assert_eq!(blocks[0].content, "fn hello() {\n    println!(\"hi\");\n}");
        assert_eq!(blocks[0].start_line, 1);
        assert_eq!(blocks[0].end_line, 5);
    }

    #[test]
    fn detect_multiple_code_blocks() {
        let lines = vec![
            "```go",
            "package main",
            "```",
            "and",
            "```",
            "plain text",
            "```",
        ];
        let blocks = detect_code_blocks(&lines);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, "go");
        assert_eq!(blocks[1].language, "");
    }

    #[test]
    fn diff_preview_shows_changes() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nmodified\nline3\n";
        let diff = compute_diff_preview(old, new);
        let has_add = diff.iter().any(|d| d.tag == DiffTag::Add);
        let has_remove = diff.iter().any(|d| d.tag == DiffTag::Remove);
        assert!(has_add);
        assert!(has_remove);
    }
}
