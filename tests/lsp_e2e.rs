//! LSP protocol tests — initialize, shutdown, exit.

use std::path::PathBuf;
use std::time::Duration;

use kairn::lsp::client::LspClient;
use kairn::lsp::messages::LspMessage;

fn spawn_lsp() -> LspClient {
    let path = PathBuf::from(env!("CARGO_BIN_EXE_rusticle-lsp"));
    let prelude = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".kairn/prelude.tcl")
        .to_string_lossy()
        .to_string();
    let args = ["--prelude", &prelude];
    LspClient::spawn(
        path.to_str().unwrap(),
        &args,
        &std::collections::HashMap::new(),
        txv_core::run::Waker::noop(),
    )
    .expect("Failed to spawn rusticle-lsp")
}

fn poll_response(client: &mut LspClient, timeout_ms: u64) -> Option<LspMessage> {
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

fn init(client: &mut LspClient) {
    client.send_request("initialize", serde_json::json!({"capabilities": {}}));
    poll_response(client, 2000).expect("initialize response");
}

#[test]
fn e2e_lsp_initialize() {
    let mut client = spawn_lsp();
    let id = client.send_request("initialize", serde_json::json!({"capabilities": {}}));
    let msg = poll_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            assert!(r["capabilities"]["definitionProvider"].as_bool().unwrap_or(false));
            assert!(r["capabilities"]["completionProvider"].is_object());
            assert!(r["capabilities"]["hoverProvider"].as_bool().unwrap_or(false));
        }
        _ => panic!("Expected successful response"),
    }
}

#[test]
fn e2e_lsp_shutdown() {
    let mut client = spawn_lsp();
    init(&mut client);

    let id = client.send_request("shutdown", serde_json::json!(null));
    let msg = poll_response(&mut client, 2000).expect("No response");
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
    client.send_notification("exit", serde_json::json!(null));
    std::thread::sleep(Duration::from_millis(100));
    assert!(client.poll().is_empty());
}

#[test]
fn e2e_lsp_rename() {
    use kairn::lsp::requests;

    let mut client = spawn_lsp();
    init(&mut client);

    let id = requests::rename(&mut client, "file:///test.tcl", 0, 0, "new_name");
    let msg = poll_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            assert!(r.get("changes").is_some());
        }
        _ => panic!("Expected rename response"),
    }
}
