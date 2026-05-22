//! LSP scenario tests — protocol, handler routing, registry behavior.
//!
//! Tests the LSP subsystem without requiring a real language server.
//! Uses mock data and direct function calls to verify behavior.

use serde_json::json;

use kairn::lsp::messages::{self, LspMessage};
use kairn::lsp::requests;

// --- Test 1: Server start (spawn nonexistent → None) ---

#[test]
fn lsp_server_spawn_nonexistent_returns_none() {
    use kairn::lsp::client::LspClient;
    let result = LspClient::spawn("__no_such_lsp_binary__", &[], txv_core::run::Waker::noop());
    assert!(result.is_none());
}

// --- Test 2: Server fail → disabled (no retry) ---

#[test]
fn lsp_registry_disables_after_spawn_failure() {
    use kairn::lsp::registry::LspRegistry;
    use std::path::Path;

    let mut reg = LspRegistry::new();
    reg.set_config("testlang", "__no_such_binary__", &[]);

    // First attempt: fails, should disable
    let result = reg.ensure_started("testlang", Path::new("/tmp"));
    assert!(!result);

    // Second attempt: should not retry (disabled)
    let result = reg.ensure_started("testlang", Path::new("/tmp"));
    assert!(!result);
}

// --- Test 3: goto definition response parsing ---

#[test]
fn lsp_goto_def_parses_single_location() {
    let result = json!({
        "uri": "file:///home/user/src/main.rs",
        "range": {
            "start": {"line": 42, "character": 4},
            "end": {"line": 42, "character": 12}
        }
    });
    let locs = requests::parse_locations(&result);
    assert_eq!(locs.len(), 1);
    assert_eq!(locs[0].uri, "file:///home/user/src/main.rs");
    assert_eq!(locs[0].line, 42);
    assert_eq!(locs[0].character, 4);
}

#[test]
fn lsp_goto_def_parses_location_link_array() {
    let result = json!([
        {
            "uri": "file:///a.rs",
            "range": {"start": {"line": 10, "character": 0}, "end": {"line": 10, "character": 5}}
        },
        {
            "uri": "file:///b.rs",
            "range": {"start": {"line": 20, "character": 3}, "end": {"line": 20, "character": 7}}
        }
    ]);
    let locs = requests::parse_locations(&result);
    assert_eq!(locs.len(), 2);
    assert_eq!(locs[0].line, 10);
    assert_eq!(locs[1].uri, "file:///b.rs");
}

#[test]
fn lsp_goto_def_null_result_empty() {
    let result = json!(null);
    let locs = requests::parse_locations(&result);
    assert!(locs.is_empty());
}

// --- Test 4: find references response parsing ---

#[test]
fn lsp_find_refs_parses_multiple_locations() {
    let result = json!([
        {"uri": "file:///src/lib.rs", "range": {"start": {"line": 5, "character": 0}, "end": {"line": 5, "character": 3}}},
        {"uri": "file:///src/main.rs", "range": {"start": {"line": 12, "character": 4}, "end": {"line": 12, "character": 7}}},
        {"uri": "file:///tests/test.rs", "range": {"start": {"line": 1, "character": 8}, "end": {"line": 1, "character": 11}}}
    ]);
    let locs = requests::parse_locations(&result);
    assert_eq!(locs.len(), 3);
    assert_eq!(locs[2].uri, "file:///tests/test.rs");
    assert_eq!(locs[2].line, 1);
}

// --- Test 5: hover response parsing ---

#[test]
fn lsp_hover_parses_string_contents() {
    let result = json!({"contents": "fn main() -> ()"});
    let text = requests::parse_hover(&result);
    assert_eq!(text.as_deref(), Some("fn main() -> ()"));
}

#[test]
fn lsp_hover_parses_markup_content() {
    let result = json!({"contents": {"kind": "markdown", "value": "```rust\nfn foo()\n```"}});
    let text = requests::parse_hover(&result);
    assert_eq!(text.as_deref(), Some("```rust\nfn foo()\n```"));
}

#[test]
fn lsp_hover_parses_array_contents() {
    let result = json!({"contents": ["line1", {"language": "rust", "value": "line2"}]});
    let text = requests::parse_hover(&result);
    assert_eq!(text.as_deref(), Some("line1\nline2"));
}

#[test]
fn lsp_hover_null_returns_none() {
    let result = json!(null);
    let text = requests::parse_hover(&result);
    assert!(text.is_none());
}

// --- Test 6: completion response parsing ---

#[test]
fn lsp_completion_parses_items_array() {
    let result = json!({
        "items": [
            {"label": "println!", "detail": "macro", "insertText": "println!($0)"},
            {"label": "print!", "detail": "macro"}
        ]
    });
    let items = requests::parse_completion(&result);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].label, "println!");
    assert_eq!(items[0].insert_text.as_deref(), Some("println!($0)"));
    assert_eq!(items[1].detail.as_deref(), Some("macro"));
    assert!(items[1].insert_text.is_none());
}

#[test]
fn lsp_completion_parses_flat_array() {
    let result = json!([
        {"label": "foo"},
        {"label": "bar"}
    ]);
    let items = requests::parse_completion(&result);
    assert_eq!(items.len(), 2);
    assert_eq!(items[1].label, "bar");
}

#[test]
fn lsp_completion_empty_on_null() {
    let result = json!(null);
    let items = requests::parse_completion(&result);
    assert!(items.is_empty());
}

// --- Test 7: didChange — verify encode produces valid JSON-RPC ---

#[test]
fn lsp_did_change_encodes_correctly() {
    let data = messages::encode_notification(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": "file:///test.rs", "version": 2},
            "contentChanges": [{"text": "fn main() {}"}]
        }),
    );
    let s = String::from_utf8(data).unwrap();
    assert!(s.starts_with("Content-Length: "));
    assert!(s.contains("textDocument/didChange"));
    assert!(s.contains("\"version\":2"));
    assert!(s.contains("fn main() {}"));
}

// --- Test 8: diagnostics notification parsing ---

#[test]
fn lsp_diagnostics_notification_parsed() {
    let json = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": "file:///src/main.rs",
            "diagnostics": [
                {
                    "range": {"start": {"line": 3, "character": 0}, "end": {"line": 3, "character": 10}},
                    "severity": 1,
                    "message": "unused variable"
                }
            ]
        }
    });
    let msg = messages::parse_message(&json);
    match msg {
        Some(LspMessage::Notification { method, params }) => {
            assert_eq!(method, "textDocument/publishDiagnostics");
            let uri = params["uri"].as_str().unwrap();
            assert_eq!(uri, "file:///src/main.rs");
            let diags = params["diagnostics"].as_array().unwrap();
            assert_eq!(diags.len(), 1);
            assert_eq!(diags[0]["message"], "unused variable");
        }
        _ => panic!("expected notification"),
    }
}

// --- Test 9: server crash — graceful handling ---

#[test]
fn lsp_client_poll_empty_after_process_exit() {
    use kairn::lsp::client::LspClient;
    // Spawn `true` which exits immediately
    let client = LspClient::spawn("true", &[], txv_core::run::Waker::noop());
    if let Some(mut c) = client {
        // Give it a moment to exit
        std::thread::sleep(std::time::Duration::from_millis(50));
        // Poll should return empty, not panic
        let msgs = c.poll();
        assert!(msgs.is_empty());
    }
}

// --- Test 10: disabled after fail — no retry ---

#[test]
fn lsp_registry_no_retry_after_disable() {
    use kairn::lsp::registry::LspRegistry;
    use std::path::Path;

    let mut reg = LspRegistry::new();
    reg.set_config("fakeland", "__nonexistent__", &[]);

    // First call: spawn fails → auto-disabled
    assert!(!reg.ensure_started("fakeland", Path::new("/tmp")));

    // Override with a valid command — but it's disabled, so still false
    reg.set_config("fakeland", "echo", &[]);
    assert!(!reg.ensure_started("fakeland", Path::new("/tmp")));
}

// --- Test: code actions parsing ---

#[test]
fn lsp_code_actions_parses_titles() {
    let result = json!([
        {"title": "Extract function", "kind": "refactor.extract"},
        {"title": "Add import", "kind": "quickfix"}
    ]);
    let actions = requests::parse_code_actions(&result);
    assert_eq!(actions, vec!["Extract function", "Add import"]);
}

#[test]
fn lsp_code_actions_empty_on_null() {
    let result = json!(null);
    let actions = requests::parse_code_actions(&result);
    assert!(actions.is_empty());
}
