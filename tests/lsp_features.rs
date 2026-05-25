//! LSP language feature tests — diagnostics, completion, hover, definition, references.

use std::path::PathBuf;
use std::time::Duration;

use kairn::lsp::client::LspClient;
use kairn::lsp::messages::LspMessage;
use kairn::lsp::requests;

fn spawn_lsp() -> LspClient {
    let path = PathBuf::from(env!("CARGO_BIN_EXE_rusticle-lsp"));
    let prelude = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".kairn/prelude.tcl")
        .to_string_lossy()
        .to_string();
    let args = ["--prelude", &prelude];
    LspClient::spawn(path.to_str().unwrap(), &args, txv_core::run::Waker::noop()).expect("Failed to spawn rusticle-lsp")
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

fn poll_notification(client: &mut LspClient, method: &str, timeout_ms: u64) -> Option<LspMessage> {
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let msgs = client.poll();
        for msg in msgs {
            if let LspMessage::Notification { method: m, .. } = &msg {
                if m == method {
                    return Some(msg);
                }
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

fn open_tcl(client: &mut LspClient, uri: &str, text: &str) {
    client.send_notification(
        "textDocument/didOpen",
        serde_json::json!({
            "textDocument": {"uri": uri, "languageId": "tcl", "version": 1, "text": text}
        }),
    );
    poll_notification(client, "textDocument/publishDiagnostics", 500);
}

#[test]
fn e2e_lsp_diagnostics_on_open() {
    let mut client = spawn_lsp();
    init(&mut client);

    client.send_notification(
        "textDocument/didOpen",
        serde_json::json!({
            "textDocument": {
                "uri": "file:///test.tcl", "languageId": "tcl",
                "version": 1, "text": "editor open foo\nbadcmd hello\n"
            }
        }),
    );

    let msg = poll_notification(&mut client, "textDocument/publishDiagnostics", 2000).expect("No diagnostics");
    if let LspMessage::Notification { params, .. } = msg {
        let diags = params["diagnostics"].as_array().expect("diagnostics array");
        assert_eq!(diags.len(), 1);
        assert!(diags[0]["message"].as_str().unwrap_or("").contains("badcmd"));
        assert_eq!(diags[0]["range"]["start"]["line"], 1);
    }
}

#[test]
fn e2e_lsp_completion() {
    let mut client = spawn_lsp();
    init(&mut client);
    open_tcl(&mut client, "file:///test.tcl", "edi");

    let id = requests::completion(&mut client, "file:///test.tcl", 0, 3);
    let msg = poll_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let items = requests::parse_completion(&r);
            let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
            assert!(labels.contains(&"editor"), "should complete 'editor': {labels:?}");
        }
        _ => panic!("Expected completion response"),
    }
}

#[test]
fn e2e_lsp_hover_builtin() {
    let mut client = spawn_lsp();
    init(&mut client);
    open_tcl(&mut client, "file:///test.tcl", "set x 42\nputs $x\n");

    let id = requests::hover(&mut client, "file:///test.tcl", 0, 1);
    let msg = poll_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let text = requests::parse_hover(&r).expect("hover text");
            assert!(text.contains("set"), "hover should mention 'set': {text}");
        }
        _ => panic!("Expected hover response"),
    }
}

#[test]
fn e2e_lsp_goto_definition() {
    let mut client = spawn_lsp();
    init(&mut client);
    open_tcl(
        &mut client,
        "file:///test.tcl",
        "proc greet {name} { puts $name }\ngreet world\n",
    );

    let id = requests::goto_definition(&mut client, "file:///test.tcl", 1, 2);
    let msg = poll_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let locs = requests::parse_locations(&r);
            assert_eq!(locs.len(), 1);
            assert_eq!(locs[0].line, 0);
        }
        _ => panic!("Expected definition response"),
    }
}

#[test]
fn e2e_lsp_find_references() {
    let mut client = spawn_lsp();
    init(&mut client);
    open_tcl(
        &mut client,
        "file:///test.tcl",
        "proc greet {name} { puts $name }\ngreet world\ngreet again\n",
    );

    let id = requests::find_references(&mut client, "file:///test.tcl", 1, 2);
    let msg = poll_response(&mut client, 2000).expect("No response");
    match msg {
        LspMessage::Response {
            id: rid,
            result: Some(r),
            ..
        } => {
            assert_eq!(rid, id);
            let locs = requests::parse_locations(&r);
            assert!(locs.len() >= 2, "expected at least 2 references, got {}", locs.len());
        }
        _ => panic!("Expected references response"),
    }
}

#[test]
fn e2e_lsp_shebang_preamble_suppresses_unknown_command() {
    // Spawn LSP WITHOUT --prelude to test shebang-based discovery
    let path = PathBuf::from(env!("CARGO_BIN_EXE_rusticle-lsp"));
    let mut client = LspClient::spawn(path.to_str().unwrap(), &[], txv_core::run::Waker::noop())
        .expect("Failed to spawn rusticle-lsp");
    init(&mut client);

    // File with rusticle-tk shebang — "app" should be recognized via --lsp-preamble
    let text = "#!/usr/bin/env rusticle-tk\napp run\n";
    client.send_notification(
        "textDocument/didOpen",
        serde_json::json!({
            "textDocument": {
                "uri": "file:///shebang_test.tcl", "languageId": "tcl",
                "version": 1, "text": text
            }
        }),
    );

    let msg = poll_notification(&mut client, "textDocument/publishDiagnostics", 3000).expect("No diagnostics");
    if let LspMessage::Notification { params, .. } = msg {
        let diags = params["diagnostics"].as_array().expect("diagnostics array");
        // "app" should NOT be flagged as unknown (preamble registered it)
        let has_app_error = diags
            .iter()
            .any(|d| d["message"].as_str().unwrap_or("").contains("app") && d["severity"].as_u64() == Some(1));
        assert!(
            !has_app_error,
            "shebang preamble should suppress 'app' error: {diags:?}"
        );
    }
}
