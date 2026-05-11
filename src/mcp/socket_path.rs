//! Socket path computation for the MCP server.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// Compute the Unix socket path: `$XDG_RUNTIME_DIR/kairn-{hash}.sock`
/// where hash = first 8 hex chars of SHA256(canonical root path).
pub fn socket_path(root: &Path) -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
    let hash = {
        let mut hasher = Sha256::new();
        hasher.update(root.to_string_lossy().as_bytes());
        let result = hasher.finalize();
        hex_prefix(&result)
    };
    PathBuf::from(dir).join(format!("kairn-{hash}.sock"))
}

fn hex_prefix(bytes: &[u8]) -> String {
    bytes.iter().take(4).map(|b| format!("{b:02x}")).collect()
}
