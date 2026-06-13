//! Test: MCP DiffRevert action reverts a hunk in diff mode.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use txv_core::event::{KeyCode, KeyMod};
use txv_core::run::Waker;

fn git_project(filename: &str, content: &str) -> tempfile::TempDir {
    let dir = temp_project(&[(filename, content)]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
    dir
}

fn exec_mcp(h: &mut TestHarness, action: McpAction) -> Result<serde_json::Value, String> {
    let queue = h.state.mcp_commands().as_ref().unwrap();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    if let Ok(mut q) = queue.queue_handle().lock() {
        q.push_back(McpRequest::new(action, tx));
    }
    h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    rx.recv().map_err(|e| e.to_string())?
}

#[test]
fn mcp_diff_revert_reverts_hunk() {
    // Make the change on line 1 so the diff starts with a change at cursor=0
    let dir = git_project("x.txt", "aaa\nbbb\nccc\n");
    std::fs::write(dir.path().join("x.txt"), "AAA\nbbb\nccc\n").unwrap();

    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    h.run_cycles(2);

    // Open file (same pattern as diff_revert_scenario.rs)
    let path = h.state.root_dir().join("x.txt");
    let req = OpenFileRequest::new(path);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(2);

    // Enter diff mode: type :diff<Enter>
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("diff");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(4);

    // Dispatch MCP DiffRevert (cursor=0 is on the change since first line differs)
    let result = exec_mcp(
        &mut h,
        McpAction::DiffRevert {
            name: "x.txt".to_string(),
        },
    );
    assert!(result.is_ok(), "DiffRevert failed: {result:?}");
    h.run_cycles(2);

    // Save via MCP (DiffView tab may be active, so :w won't reach the editor)
    let save = exec_mcp(
        &mut h,
        McpAction::SaveFile {
            name: "x.txt".to_string(),
        },
    );
    assert!(save.is_ok(), "SaveFile failed: {save:?}");

    let content = std::fs::read_to_string(dir.path().join("x.txt")).unwrap();
    assert_eq!(content, "aaa\nbbb\nccc\n", "file should be reverted to original");
}
