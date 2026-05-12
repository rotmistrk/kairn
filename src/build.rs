//! Build/run/test command execution and error parsing.

use std::path::Path;
use std::process::Command;

/// A parsed error location from build output.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorLocation {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub message: String,
}

/// Default build command for a language.
pub fn default_build_command(lang: &str) -> &'static str {
    match lang {
        "rust" => "cargo build",
        "go" => "go build ./...",
        "typescript" | "javascript" => "npm run build",
        "c" | "cpp" => "make",
        "java" => "mvn compile",
        "python" => "python -m py_compile",
        _ => "make",
    }
}

/// Default run command for a language.
pub fn default_run_command(lang: &str) -> &'static str {
    match lang {
        "rust" => "cargo run",
        "go" => "go run .",
        "typescript" | "javascript" => "npm start",
        "java" => "mvn exec:java",
        "python" => "python main.py",
        _ => "make run",
    }
}

/// Default test command for a language.
pub fn default_test_command(lang: &str) -> &'static str {
    match lang {
        "rust" => "cargo test",
        "go" => "go test ./...",
        "typescript" | "javascript" => "npm test",
        "java" => "mvn test",
        "python" => "pytest",
        _ => "make test",
    }
}

/// Run a shell command and capture output. Non-blocking via thread.
pub fn run_command(cmd: &str, cwd: &Path) -> Option<String> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    let output = match Command::new("sh").arg("-c").arg(cmd).current_dir(cwd).output() {
        Ok(o) => o,
        Err(e) => {
            log::error!("build: failed to spawn '{cmd}': {e}");
            return Some(format!("Build failed: {e}"));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };
    Some(combined)
}

/// Parse error locations from build output (supports common formats).
pub fn parse_errors(output: &str) -> Vec<ErrorLocation> {
    let mut errors = Vec::new();
    for line in output.lines() {
        if let Some(loc) = parse_gcc_style(line) {
            errors.push(loc);
        } else if let Some(loc) = parse_rust_style(line) {
            errors.push(loc);
        }
    }
    errors
}

/// Parse "file:line:col: message" (gcc, clang, go, typescript)
fn parse_gcc_style(line: &str) -> Option<ErrorLocation> {
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() < 4 {
        return None;
    }
    let file = parts[0].trim();
    let line_num: u32 = parts[1].trim().parse().ok()?;
    let col: u32 = parts[2].trim().parse().ok()?;
    let message = parts[3].trim().to_string();
    // Skip if file doesn't look like a path
    if file.is_empty() || file.starts_with(' ') {
        return None;
    }
    Some(ErrorLocation {
        file: file.to_string(),
        line: line_num,
        col,
        message,
    })
}

/// Parse Rust-style "  --> file:line:col"
fn parse_rust_style(line: &str) -> Option<ErrorLocation> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("-->")?;
    let rest = rest.trim();
    let parts: Vec<&str> = rest.splitn(3, ':').collect();
    if parts.len() < 3 {
        return None;
    }
    let file = parts[0].trim();
    let line_num: u32 = parts[1].trim().parse().ok()?;
    let col: u32 = parts[2].trim().parse().ok()?;
    Some(ErrorLocation {
        file: file.to_string(),
        line: line_num,
        col,
        message: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_gcc_error() {
        let line = "src/main.rs:10:5: error: unused variable";
        let errors = parse_errors(line);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "src/main.rs");
        assert_eq!(errors[0].line, 10);
        assert_eq!(errors[0].col, 5);
    }

    #[test]
    fn parse_rust_error() {
        let line = "  --> src/lib.rs:42:9";
        let errors = parse_errors(line);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].file, "src/lib.rs");
        assert_eq!(errors[0].line, 42);
        assert_eq!(errors[0].col, 9);
    }

    #[test]
    fn parse_no_errors() {
        let output = "Compiling kairn v0.1.0\nFinished dev";
        let errors = parse_errors(output);
        assert!(errors.is_empty());
    }

    #[test]
    fn default_commands_exist() {
        assert_eq!(default_build_command("rust"), "cargo build");
        assert_eq!(default_run_command("go"), "go run .");
        assert_eq!(default_test_command("python"), "pytest");
    }
}
