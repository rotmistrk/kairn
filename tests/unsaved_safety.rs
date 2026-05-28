//! Tests for unsaved-file safety: Alt-w, Ctrl-Q, save-all, disk-change.

mod helpers;

use helpers::TestHarness;
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use tempfile::TempDir;
use txv_core::event::{KeyCode, KeyMod};

fn temp_project(files: &[(&str, &str)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, content) in files {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    dir
}

fn open_file(h: &mut TestHarness, name: &str) {
    let path = h.state.root_dir().join(name);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(OpenFileRequest::new(path))));
}

/// Disable autosave so dirty buffers trigger the confirm prompt.
fn disable_autosave(h: &mut TestHarness) {
    h.state.settings_mut().editor_defaults_mut().set_autosave(false);
}

fn alt(code: KeyCode) -> (KeyCode, KeyMod) {
    (
        code,
        KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    )
}

// --- Task 1: Alt-w must prompt on dirty buffer ---

#[test]
fn alt_w_prompts_on_dirty_buffer() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    disable_autosave(&mut h);
    open_file(&mut h, "a.rs");
    h.run_cycles(2);

    // Edit the file to make it dirty
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Alt-w should NOT close — should prompt
    let (code, mods) = alt(KeyCode::Char('w'));
    h.inject_key(code, mods);
    h.run_cycles(2);

    // Editor should still be open (tab not closed)
    assert!(
        h.content_contains("X"),
        "dirty buffer should still be visible after Alt-w"
    );
}

#[test]
fn alt_w_closes_clean_buffer() {
    let dir = temp_project(&[("a.rs", "hello"), ("b.rs", "world")]);
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "a.rs");
    open_file(&mut h, "b.rs");
    h.run_cycles(2);

    // b.rs is active and clean — Alt-w should close it
    let (code, mods) = alt(KeyCode::Char('w'));
    h.inject_key(code, mods);
    h.run_cycles(2);

    // Should now show a.rs (b.rs closed)
    assert!(h.content_contains("hello"), "should show a.rs after closing b.rs");
}

#[test]
fn alt_w_save_and_close_dirty() {
    let dir = temp_project(&[("a.rs", "original")]);
    let mut h = TestHarness::new(dir.path());
    disable_autosave(&mut h);
    open_file(&mut h, "a.rs");
    h.run_cycles(2);

    // Edit
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('Z'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);

    // Alt-w triggers prompt
    let (code, mods) = alt(KeyCode::Char('w'));
    h.inject_key(code, mods);
    h.run_cycles(2);

    // Press 'y' to save and close
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(2);

    // File should be saved
    let content = std::fs::read_to_string(dir.path().join("a.rs")).unwrap();
    assert!(content.contains('Z'), "file should be saved after Alt-w + y");
}

// --- Task 2: Ctrl-Q must check all tabs for unsaved changes ---

#[test]
fn ctrl_q_blocked_by_dirty_buffer() {
    let dir = temp_project(&[("a.rs", "hello"), ("b.rs", "world")]);
    let mut h = TestHarness::new(dir.path());
    disable_autosave(&mut h);
    open_file(&mut h, "a.rs");
    open_file(&mut h, "b.rs");
    h.run_cycles(2);

    // Edit b.rs to make it dirty
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('Z'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Ctrl-Q should NOT quit — should prompt about unsaved files
    let ctrl = KeyMod {
        ctrl: true,
        alt: false,
        shift: false,
    };
    h.inject_key(KeyCode::Char('q'), ctrl);
    h.run_cycles(2);

    // Program should still be running (not quit)
    assert!(!h.program.should_quit(), "should not quit with dirty buffer");
}

#[test]
fn ctrl_q_quits_when_all_clean() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    disable_autosave(&mut h);
    open_file(&mut h, "a.rs");
    h.run_cycles(2);

    // Ctrl-Q should quit (no dirty buffers)
    let ctrl = KeyMod {
        ctrl: true,
        alt: false,
        shift: false,
    };
    h.inject_key(KeyCode::Char('q'), ctrl);
    h.run_cycles(2);

    assert!(h.program.should_quit(), "should quit when all buffers are clean");
}

// --- Task 3: save-all must save all dirty editor tabs ---

#[test]
fn save_all_saves_all_dirty_tabs() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::new(dir.path());
    disable_autosave(&mut h);
    open_file(&mut h, "a.rs");
    h.run_cycles(2);

    // Edit a.rs
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Open and edit b.rs
    open_file(&mut h, "b.rs");
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('Y'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Trigger save-all via command
    h.dispatch_command(kairn::commands::CM_SAVE_ALL, None);
    h.run_cycles(2);

    // Both files should be saved
    let a = std::fs::read_to_string(dir.path().join("a.rs")).unwrap();
    let b = std::fs::read_to_string(dir.path().join("b.rs")).unwrap();
    assert!(a.contains('X'), "a.rs should be saved: {a:?}");
    assert!(b.contains('Y'), "b.rs should be saved: {b:?}");
}

// --- Task 4: Disk change detection ---

#[test]
fn disk_change_auto_reloads_clean_buffer() {
    let dir = temp_project(&[("a.rs", "original")]);
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "a.rs");
    h.run_cycles(2);

    // Verify original content
    assert!(h.content_contains("original"), "should show original content");

    // Modify file on disk externally
    std::thread::sleep(std::time::Duration::from_millis(50));
    std::fs::write(dir.path().join("a.rs"), "reloaded").unwrap();

    // Run enough ticks for check_disk_change to fire (every 20 ticks)
    h.run_cycles(25);

    // Buffer should auto-reload with new content
    assert!(h.content_contains("reloaded"), "should auto-reload clean buffer");
}

#[test]
fn disk_change_prompts_on_dirty_buffer() {
    let dir = temp_project(&[("a.rs", "original")]);
    let mut h = TestHarness::new(dir.path());
    disable_autosave(&mut h);
    open_file(&mut h, "a.rs");
    h.run_cycles(2);

    // Edit the buffer to make it dirty
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('Z'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Modify file on disk externally
    std::thread::sleep(std::time::Duration::from_millis(50));
    std::fs::write(dir.path().join("a.rs"), "external change").unwrap();

    // Run enough ticks for check_disk_change to fire
    h.run_cycles(25);

    // Should show prompt (confirm item visible), buffer should still have our edit
    assert!(h.content_contains("Z"), "dirty buffer should keep local edits");
    // The confirm prompt is in the status bar (last row)
    assert!(
        h.contains("changed on disk") || h.contains("reload"),
        "should show reload prompt in status bar"
    );
}

// --- Race condition: autosave + quit ---

#[test]
fn ctrl_q_saves_autosave_pending_buffers() {
    let dir = temp_project(&[("a.rs", "original")]);
    let mut h = TestHarness::new(dir.path());
    // autosave is ON by default — don't disable it
    open_file(&mut h, "a.rs");
    h.run_cycles(2);

    // Edit the buffer (dirty, but autosave hasn't fired yet)
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('Q'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Ctrl-Q immediately (don't wait for autosave tick)
    let ctrl = KeyMod {
        ctrl: true,
        alt: false,
        shift: false,
    };
    h.inject_key(KeyCode::Char('q'), ctrl);
    h.run_cycles(2);

    // Program should quit (autosave means no prompt)
    assert!(h.program.should_quit(), "should quit with autosave enabled");

    // File should be saved (the race fix: save before quit)
    let content = std::fs::read_to_string(dir.path().join("a.rs")).unwrap();
    assert!(
        content.contains('Q'),
        "autosave buffer should be flushed on quit: {content:?}"
    );
}

#[test]
fn alt_w_saves_autosave_pending_buffer_before_close() {
    let dir = temp_project(&[("a.rs", "original"), ("b.rs", "keep")]);
    let mut h = TestHarness::new(dir.path());
    // autosave ON (default)
    open_file(&mut h, "a.rs");
    open_file(&mut h, "b.rs");
    h.run_cycles(2);

    // Switch to a.rs and edit it
    let (code, mods) = alt(KeyCode::Char(';'));
    h.inject_key(code, mods);
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('W'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Alt-w closes immediately (autosave = no prompt)
    let (code, mods) = alt(KeyCode::Char('w'));
    h.inject_key(code, mods);
    h.run_cycles(2);

    // File should be saved before close
    let content = std::fs::read_to_string(dir.path().join("a.rs")).unwrap();
    assert!(
        content.contains('W'),
        "autosave buffer should be flushed on Alt-w: {content:?}"
    );
}
