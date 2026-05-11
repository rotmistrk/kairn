//! Append-only MCP diagnostics log.
//!
//! Writes to `$XDG_RUNTIME_DIR/kairn-mcp.log`.

use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

fn log_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    PathBuf::from(dir).join("kairn-mcp.log")
}

/// Append a timestamped log line. Silently ignores failures.
pub fn log(component: &str, msg: &str) {
    let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log_path()) else {
        return;
    };
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let pid = std::process::id();
    let _ = writeln!(f, "{ts} [{pid}] {component}: {msg}");
}
