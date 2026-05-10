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
}

impl LspClient {
    /// Spawn an LSP server process. Returns None if spawn fails.
    pub fn spawn(cmd: &str, args: &[&str]) -> Option<Self> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;

        let write_tx = Self::start_writer(stdin);
        let msg_rx = Self::start_reader(stdout);

        Some(Self {
            next_id: 1,
            write_tx,
            msg_rx,
            child,
        })
    }

    /// Send a request. Returns the request id.
    pub fn send_request(&mut self, method: &str, params: Value) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let data = messages::encode_request(id, method, params);
        let _ = self.write_tx.send(data);
        id
    }

    /// Send a notification (no response expected).
    pub fn send_notification(&mut self, method: &str, params: Value) {
        let data = messages::encode_notification(method, params);
        let _ = self.write_tx.send(data);
    }

    /// Poll for incoming messages (non-blocking). Returns all available.
    pub fn poll(&self) -> Vec<LspMessage> {
        let mut msgs = Vec::new();
        while let Ok(msg) = self.msg_rx.try_recv() {
            msgs.push(msg);
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
            loop {
                match read_message(&mut reader) {
                    Some(msg) => {
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                    None => break,
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
        if reader.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some(len_str) = line.strip_prefix("Content-Length: ") {
            content_length = len_str.parse().ok()?;
        }
    }
    if content_length == 0 {
        return None;
    }
    let mut body = vec![0u8; content_length];
    std::io::Read::read_exact(reader, &mut body).ok()?;
    let json: Value = serde_json::from_slice(&body).ok()?;
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
        if let Some(c) = client {
            let msgs = c.poll();
            assert!(msgs.is_empty());
        }
    }
}
