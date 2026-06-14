//! Unix socket listener for the MCP server.

use std::fs;
use std::io::{BufReader, BufWriter};
use std::os::unix::net::{UnixListener, UnixStream};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::commands::McpCommandQueue;
use super::log;
use super::permissions::PermissionHandle;
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
    permissions: Option<PermissionHandle>,
    socket_path: &Path,
) -> Result<PathBuf, String> {
    log::log("listener", &format!("binding {}", socket_path.display()));
    check_no_existing_instance(socket_path)?;
    let listener = bind_listener(socket_path)?;

    log::log("listener", "accepting connections");

    let path = socket_path.to_path_buf();
    thread::spawn(move || accept_loop(listener, snapshot, cmd_queue, permissions));
    Ok(path)
}

fn check_no_existing_instance(socket_path: &Path) -> Result<(), String> {
    if UnixStream::connect(socket_path).is_ok() {
        return Err("kairn already running for this project".to_owned());
    }
    if socket_path.exists() {
        let _ = fs::remove_file(socket_path);
    }
    Ok(())
}

fn bind_listener(socket_path: &Path) -> Result<UnixListener, String> {
    let listener = UnixListener::bind(socket_path).map_err(|e| format!("Failed to bind MCP socket: {e}"))?;
    listener
        .set_nonblocking(false)
        .map_err(|e| format!("Failed to set socket blocking: {e}"))?;
    Ok(listener)
}

fn accept_loop(
    listener: UnixListener,
    snapshot: Arc<Mutex<McpSnapshot>>,
    cmd_queue: SharedCommandQueue,
    permissions: Option<PermissionHandle>,
) {
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
        let writer = BufWriter::new(stream);

        let snap = Arc::clone(&snapshot);
        let cq = cmd_queue.lock().ok().and_then(|g| g.clone());
        let perms = permissions.clone();
        thread::spawn(move || handle_connection(snap, cq, perms, reader, writer));
    }
}

fn handle_connection(
    snap: Arc<Mutex<McpSnapshot>>,
    cq: Option<McpCommandQueue>,
    perms: Option<PermissionHandle>,
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
) {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let server = McpServer::new(snap, cq, perms);
        server.run(reader, writer)
    }));
    match result {
        Ok(Ok(())) => log::log("listener", "connection closed normally"),
        Ok(Err(e)) => log::log("listener", &format!("connection error: {e}")),
        Err(_) => log::log("listener", "connection handler panicked"),
    }
}
