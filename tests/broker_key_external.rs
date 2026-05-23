//! Regression test: opening a file outside root_dir must use the same broker key
//! as session restore would. A mismatch causes duplicate tabs and layout corruption.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use kairn::desktop::SlotId;

#[test]
fn external_file_broker_key_matches_session_path() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    // Create a file outside the project root (simulates cargo registry)
    let external_dir = tempfile::tempdir().unwrap();
    let ext_file = external_dir.path().join("style.rs");
    std::fs::write(&ext_file, "pub struct Style;\n").unwrap();

    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open the external file (simulates gd → LSP goto definition)
    let req = OpenFileRequest::at(ext_file.clone(), 0, 0);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // The tab should be open with the external file content
    assert!(h.content_contains("pub struct Style"));

    // Now simulate what session restore does: broker.open with the stored path.
    // Session stores: path.strip_prefix(root).unwrap_or(path) — for external files, full path.
    let session_key = ext_file.to_string_lossy().to_string();
    let result = h.state.broker.open(&session_key, SlotId::Center, 0);

    // Must be AlreadyOpen — not Opened (which would mean key mismatch)
    assert!(
        matches!(result, kairn::broker::OpenResult::AlreadyOpen { .. }),
        "Broker key mismatch: session would use {:?} but file was registered with a different key",
        session_key
    );
}
