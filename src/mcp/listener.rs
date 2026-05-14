//! Unix socket listener for the MCP server.

use std::io::BufReader;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::commands::McpCommandQueue;
use super::log;
use super::server::McpServer;
use super::snapshot::McpSnapshot;

/// Shared handle for the MCP command queue (set after waker is available).
pub type SharedCommandQueue = Arc<Mutex<Option<McpCommandQueue>>>;

/// Start the MCP listener thread on the given socket path.
///
/// # Errors
/// Returns an error string if binding fails.
pub fn start_mcp_listener(
    snapshot: Arc<Mutex<McpSnapshot>>,
    cmd_queue: SharedCommandQueue,
    socket_path: &Path,
) -> Result<PathBuf, String> {
    log::log("listener", &format!("binding {}", socket_path.display()));

    // Instance lock: if we can connect, another instance is running.
    if std::os::unix::net::UnixStream::connect(socket_path).is_ok() {
        return Err("kairn already running for this project".to_owned());
    }

    // Remove stale socket file.
    if socket_path.exists() {
        let _ = std::fs::remove_file(socket_path);
    }

    let listener = UnixListener::bind(socket_path).map_err(|e| format!("Failed to bind MCP socket: {e}"))?;

    listener
        .set_nonblocking(false)
        .map_err(|e| format!("Failed to set socket blocking: {e}"))?;

    log::log("listener", "accepting connections");

    let path = socket_path.to_path_buf();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else {
                break;
            };

            log::log("listener", "client connected");

            let _ = stream.set_read_timeout(None);
            let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

            let reader = BufReader::new(match stream.try_clone() {
                Ok(s) => s,
                Err(_) => continue,
            });
            let writer = std::io::BufWriter::new(stream);

            let snap = Arc::clone(&snapshot);
            let cq = cmd_queue.lock().ok().and_then(|g| g.clone());
            std::thread::spawn(move || {
                let server = McpServer::new(snap, cq);
                let result = server.run(reader, writer);
                log::log("listener", &format!("connection closed: {result:?}"));
            });
        }
    });

    Ok(path)
}
