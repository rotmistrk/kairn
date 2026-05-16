//! End-to-end LSP tests using the mock_lsp server binary.
//!
//! These tests verify the full flow: LspClient spawns mock_lsp,
//! sends requests, receives responses, and parses them correctly.

use std::path::PathBuf;
use std::time::Duration;

use kairn::lsp::client::LspClient;
use kairn::lsp::messages::LspMessage;
use kairn::lsp::requests;

fn mock_lsp_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_mock_lsp"));
    if !path.exists() {
        // Fallback for different build profiles
        path = PathBuf::from("target/debug/mock_lsp");
    }
    path
}

fn spawn_mock() -> LspClient {
    let path = mock_lsp_path();
    LspClient::spawn(path.to_str().unwrap(), &[]).expect("Failed to spawn mock_lsp")
}

fn poll_until_response(client: &mut LspClient, timeout_ms: u64) -> Option<LspMessage> {
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let msgs = client.poll();
        for msg in msgs {
            if matches!(msg, LspMessage::Response { .. }) {
                return Some(msg);
            }
        }
        if std::time::Instant::now() > deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn e2e_lsp_initialize() {
    let mut client = spawn_mock();
    let params = serde_json::json!({
        "capabilities": {},
        "rootUri": "file:///project"
    });
    let id = client.send_request("initialize", params);
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            assert!(r["capabilities"]["definitionProvider"].as_bool().unwrap_or(false));
            assert!(r["capabilities"]["completionProvider"].is_object());
        }
        _ => panic!("Expected successful response"),
    }
}

#[test]
fn e2e_lsp_goto_definition() {
    let mut client = spawn_mock();
    // Initialize first
    client.send_request("initialize", serde_json::json!({}));
    poll_until_response(&mut client, 1000);

    let id = requests::goto_definition(&mut client, "file:///src/main.rs", 5, 10);
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let locs = requests::parse_locations(&r);
            assert_eq!(locs.len(), 1);
            assert_eq!(locs[0].uri, "file:///src/lib.rs");
            assert_eq!(locs[0].line, 10);
            assert_eq!(locs[0].character, 4);
        }
        _ => panic!("Expected definition response"),
    }
}

#[test]
fn e2e_lsp_find_references() {
    let mut client = spawn_mock();
    client.send_request("initialize", serde_json::json!({}));
    poll_until_response(&mut client, 1000);

    let id = requests::find_references(&mut client, "file:///src/main.rs", 5, 0);
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let locs = requests::parse_locations(&r);
            assert_eq!(locs.len(), 2);
            assert_eq!(locs[0].uri, "file:///src/main.rs");
            assert_eq!(locs[1].uri, "file:///src/lib.rs");
        }
        _ => panic!("Expected references response"),
    }
}

#[test]
fn e2e_lsp_hover() {
    let mut client = spawn_mock();
    client.send_request("initialize", serde_json::json!({}));
    poll_until_response(&mut client, 1000);

    let id = requests::hover(&mut client, "file:///src/main.rs", 3, 5);
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let text = requests::parse_hover(&r).expect("hover text");
            assert!(text.contains("fn hello()"));
        }
        _ => panic!("Expected hover response"),
    }
}

#[test]
fn e2e_lsp_completion() {
    let mut client = spawn_mock();
    client.send_request("initialize", serde_json::json!({}));
    poll_until_response(&mut client, 1000);

    let id = requests::completion(&mut client, "file:///src/main.rs", 8, 4);
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let items = requests::parse_completion(&r);
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].label, "println!");
            assert_eq!(items[0].insert_text.as_deref(), Some("println!($0)"));
            assert_eq!(items[1].label, "print!");
        }
        _ => panic!("Expected completion response"),
    }
}

#[test]
fn e2e_lsp_rename() {
    let mut client = spawn_mock();
    client.send_request("initialize", serde_json::json!({}));
    poll_until_response(&mut client, 1000);

    let id = requests::rename(&mut client, "file:///src/main.rs", 5, 0, "new_name");
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            // Rename returns workspace edit with changes
            assert!(r.get("changes").is_some());
        }
        _ => panic!("Expected rename response"),
    }
}

#[test]
fn e2e_lsp_did_change_tracked() {
    let mut client = spawn_mock();
    client.send_request("initialize", serde_json::json!({}));
    poll_until_response(&mut client, 1000);

    // Send didChange notifications
    client.send_notification(
        "textDocument/didChange",
        serde_json::json!({
            "textDocument": {"uri": "file:///src/main.rs", "version": 2},
            "contentChanges": [{"text": "fn main() {}"}]
        }),
    );
    client.send_notification(
        "textDocument/didChange",
        serde_json::json!({
            "textDocument": {"uri": "file:///src/main.rs", "version": 3},
            "contentChanges": [{"text": "fn main() { println!(); }"}]
        }),
    );

    // Give mock time to process
    std::thread::sleep(Duration::from_millis(50));

    // Query the mock's didChange counter
    let id = client.send_request("mock/didChangeCount", serde_json::json!({}));
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response { result: Some(r), .. } => {
            assert_eq!(r.as_u64(), Some(2));
        }
        _ => panic!("Expected count response"),
    }
    let _ = id;
}

#[test]
fn e2e_lsp_shutdown() {
    let mut client = spawn_mock();
    client.send_request("initialize", serde_json::json!({}));
    poll_until_response(&mut client, 1000);

    let id = client.send_request("shutdown", serde_json::json!(null));
    let msg = poll_until_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            assert!(r.is_null());
        }
        _ => panic!("Expected shutdown response"),
    }
    // exit notification — server should terminate
    client.send_notification("exit", serde_json::json!(null));
    std::thread::sleep(Duration::from_millis(100));
    // After exit, poll should return empty
    assert!(client.poll().is_empty());
}
