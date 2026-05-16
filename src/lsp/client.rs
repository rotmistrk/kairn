//! LspClient — spawns an LSP server process, sends requests, polls responses.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use serde_json::Value;

use super::messages::{self, LspMessage};

/// A running LSP server connection.
pub struct LspClient {
    next_id: u64,
    write_tx: Sender<Vec<u8>>,
    msg_rx: Receiver<LspMessage>,
    #[allow(dead_code)]
    child: Child,
    dead: bool,
}

impl LspClient {
    /// Spawn an LSP server process. Returns None if spawn fails.
    pub fn spawn(cmd: &str, args: &[&str]) -> Option<Self> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .ok()?;

        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;

        // Log stderr in background
        if let Some(stderr) = child.stderr.take() {
            let cmd_name = cmd.to_string();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    log::warn!("LSP stderr [{cmd_name}]: {line}");
                }
            });
        }

        let write_tx = Self::start_writer(stdin);
        let msg_rx = Self::start_reader(stdout);

        Some(Self {
            next_id: 1,
            write_tx,
            msg_rx,
            child,
            dead: false,
        })
    }

    /// Send a request. Returns the request id.
    pub fn send_request(&mut self, method: &str, params: Value) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let data = messages::encode_request(id, method, params);
        if self.write_tx.send(data).is_err() {
            log::error!("LSP send_request failed: server connection lost");
            self.dead = true;
        }
        id
    }

    /// Send a notification (no response expected).
    pub fn send_notification(&mut self, method: &str, params: Value) {
        let data = messages::encode_notification(method, params);
        if self.write_tx.send(data).is_err() {
            log::error!("LSP send_notification failed: server connection lost");
            self.dead = true;
        }
    }

    /// Returns true if the server connection is still alive.
    pub fn is_alive(&self) -> bool {
        !self.dead
    }

    /// Poll for incoming messages (non-blocking). Returns all available.
    pub fn poll(&mut self) -> Vec<LspMessage> {
        let mut msgs = Vec::new();
        loop {
            match self.msg_rx.try_recv() {
                Ok(msg) => msgs.push(msg),
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.dead = true;
                    break;
                }
            }
        }
        msgs
    }

    fn start_writer(mut stdin: std::process::ChildStdin) -> Sender<Vec<u8>> {
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = mpsc::channel();
        thread::spawn(move || {
            while let Ok(data) = rx.recv() {
                if stdin.write_all(&data).is_err() {
                    break;
                }
                if stdin.flush().is_err() {
                    break;
                }
            }
        });
        tx
    }

    fn start_reader(stdout: std::process::ChildStdout) -> Receiver<LspMessage> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            while let Some(msg) = read_message(&mut reader) {
                if tx.send(msg).is_err() {
                    break;
                }
            }
        });
        rx
    }
}

/// Read one LSP message from a buffered reader (Content-Length framing).
fn read_message(reader: &mut BufReader<std::process::ChildStdout>) -> Option<LspMessage> {
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return None,
            Ok(_) => {}
            Err(e) => {
                log::warn!("LSP read header: {e}");
                return None;
            }
        }
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some(len_str) = line.strip_prefix("Content-Length: ") {
            content_length = match len_str.parse() {
                Ok(n) => n,
                Err(e) => {
                    log::warn!("LSP bad Content-Length '{len_str}': {e}");
                    return None;
                }
            };
        }
    }
    if content_length == 0 {
        return None;
    }
    let mut body = vec![0u8; content_length];
    if let Err(e) = std::io::Read::read_exact(reader, &mut body) {
        log::warn!("LSP read body ({content_length} bytes): {e}");
        return None;
    }
    let json: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("LSP parse JSON: {e}");
            return None;
        }
    };
    messages::parse_message(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_nonexistent_returns_none() {
        let client = LspClient::spawn("__nonexistent_lsp_server_xyz__", &[]);
        assert!(client.is_none());
    }

    #[test]
    fn send_request_increments_id() {
        // Use `cat` as a dummy process (won't respond but won't crash)
        let client = LspClient::spawn("cat", &[]);
        if let Some(mut c) = client {
            let id1 = c.send_request("test", serde_json::json!({}));
            let id2 = c.send_request("test", serde_json::json!({}));
            assert_eq!(id1, 1);
            assert_eq!(id2, 2);
        }
    }

    #[test]
    fn poll_empty_when_no_response() {
        let client = LspClient::spawn("cat", &[]);
        if let Some(mut c) = client {
            let msgs = c.poll();
            assert!(msgs.is_empty());
        }
    }
}
