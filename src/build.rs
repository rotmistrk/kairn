//! Build/test async runner — spawns shell command, parses errors into TaskOutput.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use txv_core::run::Waker;

use crate::task_output::TaskOutput;
use crate::views::results::ResultEntry;

/// A parsed error location from build output (kept for backward compat).
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorLocation {
    pub(crate) file: String,
    pub(crate) line: u32,
    pub(crate) col: u32,
    pub(crate) message: String,
}

/// Spawn an async build/test command. Parses output lines for errors.
/// Returns immediately; results accumulate in the returned TaskOutput.
pub fn run_async(cmd: &str, root: &Path, waker: Waker) -> Arc<TaskOutput> {
    let state = TaskOutput::new();
    let state_clone = state.clone();
    let cmd = cmd.to_string();
    let root = root.to_path_buf();

    std::thread::spawn(move || {
        run_inner(&cmd, &root, &state_clone, &waker);
    });

    state
}

fn run_inner(cmd: &str, root: &PathBuf, state: &Arc<TaskOutput>, waker: &Waker) {
    let child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            state.set_error(format!("Failed to spawn: {e}"));
            state.mark_done();
            waker.wake();
            return;
        }
    };

    collect_output(&mut child, root, state, waker);

    let exit_code = child.wait().map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
    state.set_exit_code(exit_code);
    state.mark_done();
    waker.wake();
}

#[allow(clippy::ptr_arg)]
fn collect_output(child: &mut std::process::Child, root: &PathBuf, state: &Arc<TaskOutput>, waker: &Waker) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let state2 = state.clone();
    let root2 = root.clone();
    let waker2 = waker.clone();
    let stderr_handle = stderr.map(|se| {
        std::thread::spawn(move || {
            read_lines(se, &root2, &state2, &waker2);
        })
    });

    if let Some(so) = stdout {
        read_lines(so, root, state, waker);
    }

    if let Some(h) = stderr_handle {
        let _ = h.join();
    }
}

fn read_lines<R: std::io::Read>(reader: R, root: &Path, state: &TaskOutput, waker: &Waker) {
    let reader = BufReader::new(reader);
    let mut batch: Vec<ResultEntry> = Vec::with_capacity(16);

    for line in reader.lines() {
        let Ok(line) = line else {
            break;
        };
        if let Some(entry) = crate::build_parse::parse_line(&line, root) {
            batch.push(entry);
        } else {
            // Include non-error lines as context (no file location)
            batch.push(ResultEntry {
                path: PathBuf::new(),
                line: 0,
                col: 0,
                text: line,
            });
        }
        if batch.len() >= 16 {
            state.push_entries(&mut batch);
            waker.wake();
        }
    }
    if !batch.is_empty() {
        state.push_entries(&mut batch);
        waker.wake();
    }
}

/// Resolve the build command for the workspace.
/// Priority: .kairn/init > auto-detect > None.
pub fn resolve_build_cmd(root: &Path) -> Option<String> {
    if let Some(cmd) = read_init_cmd(root, "build") {
        return Some(cmd);
    }
    crate::build_detect::detect(root).map(|bs| bs.build.to_string())
}

/// Resolve the test command for the workspace.
pub fn resolve_test_cmd(root: &Path) -> Option<String> {
    if let Some(cmd) = read_init_cmd(root, "test") {
        return Some(cmd);
    }
    crate::build_detect::detect(root).map(|bs| bs.test.to_string())
}

/// Resolve the test-file command, substituting {file}.
pub fn resolve_test_file_cmd(root: &Path, file: &str) -> Option<String> {
    if let Some(cmd) = read_init_cmd(root, "test-file") {
        return Some(cmd.replace("{file}", file));
    }
    crate::build_detect::detect(root)
        .and_then(|bs| bs.test_file)
        .map(|t| t.replace("{file}", file))
}

/// Resolve the test-at-cursor command, substituting {test_name}.
pub fn resolve_test_at_cursor_cmd(root: &Path, test_name: &str) -> Option<String> {
    if let Some(cmd) = read_init_cmd(root, "test-at-cursor") {
        return Some(cmd.replace("{test_name}", test_name));
    }
    // Fallback: cargo test for Rust, go test -run for Go
    if root.join("Cargo.toml").exists() {
        return Some(format!("cargo test {test_name}"));
    }
    if root.join("go.mod").exists() {
        return Some(format!("go test -run {test_name} ./..."));
    }
    None
}

/// Read a command from .kairn/init file.
fn read_init_cmd(root: &Path, key: &str) -> Option<String> {
    let init_path = root.join(".kairn").join("init");
    let content = std::fs::read_to_string(init_path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(cmd) = rest.strip_prefix('=') {
                return Some(cmd.trim().to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_build_from_cargo() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(resolve_build_cmd(dir.path()), Some("cargo build".to_string()));
    }

    #[test]
    fn resolve_from_init_file() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join(".kairn")).unwrap();
        std::fs::write(dir.path().join(".kairn/init"), "build = make -j8\ntest = make check\n").unwrap();
        assert_eq!(resolve_build_cmd(dir.path()), Some("make -j8".to_string()));
        assert_eq!(resolve_test_cmd(dir.path()), Some("make check".to_string()));
    }

    #[test]
    fn resolve_test_file_substitution() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir(dir.path().join(".kairn")).unwrap();
        std::fs::write(dir.path().join(".kairn/init"), "test-file = cargo test --lib {file}\n").unwrap();
        let cmd = resolve_test_file_cmd(dir.path(), "src/main.rs").unwrap();
        assert_eq!(cmd, "cargo test --lib src/main.rs");
    }

    #[test]
    fn resolve_none_when_no_markers() {
        let dir = TempDir::new().unwrap();
        assert!(resolve_build_cmd(dir.path()).is_none());
    }
}
