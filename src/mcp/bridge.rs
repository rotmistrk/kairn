//! stdio↔Unix socket bridge for MCP.
//!
//! When kairn is invoked with `--mcp-connect`, this runs instead of the TUI.
//! Bridges stdin↔socket using two threads with `io::copy`.

use std::io::{self, LineWriter};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use super::log;

/// Run the MCP bridge. Reads `KAIRN_MCP_SOCKET` env var for socket path.
///
/// # Errors
/// Returns `io::Error` on connection or I/O failure.
pub fn run_mcp_bridge() -> io::Result<()> {
    let socket_path = std::env::var("KAIRN_MCP_SOCKET")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "KAIRN_MCP_SOCKET not set"))?;

    if socket_path.is_empty() || socket_path.starts_with("${") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("KAIRN_MCP_SOCKET has invalid value: {socket_path:?}"),
        ));
    }

    log::log("bridge", &format!("connecting to {socket_path}"));

    let socket =
        UnixStream::connect(&socket_path).map_err(|e| io::Error::new(e.kind(), format!("{socket_path}: {e}")))?;

    // Write timeout for sending to kairn; no read timeout (bridge stays alive until stdin closes)
    socket.set_write_timeout(Some(Duration::from_secs(30)))?;

    log::log("bridge", "connected, starting I/O threads");

    let mut sock_w = socket.try_clone()?;
    let mut sock_r = socket.try_clone()?;
    let shutdown = socket;

    // stdin→socket: runs until Kiro closes stdin
    let t_in = std::thread::spawn(move || {
        let n = io::copy(&mut io::stdin().lock(), &mut sock_w);
        log::log("bridge", &format!("stdin→socket ended: {n:?}"));
    });

    // socket→stdout: runs until socket is shut down
    let t_out = std::thread::spawn(move || {
        let mut stdout = LineWriter::new(io::stdout().lock());
        let n = io::copy(&mut sock_r, &mut stdout);
        log::log("bridge", &format!("socket→stdout ended: {n:?}"));
    });

    // When stdin closes (Kiro disconnects), shut down the socket so t_out exits too
    let _ = t_in.join();
    log::log("bridge", "stdin closed, shutting down");
    let _ = shutdown.shutdown(Shutdown::Both);
    let _ = t_out.join();

    log::log("bridge", "exiting");
    Ok(())
}
