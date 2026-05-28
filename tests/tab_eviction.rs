//! Tests for LRU tab eviction when at max_tabs limit.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

fn open_file(h: &mut TestHarness, name: &str) {
    let path = h.state.root_dir().join(name);
    let req = OpenFileRequest::new(path);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
}

#[test]
fn clean_lru_tab_evicted_when_at_limit() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    h.state.settings_mut().set_max_tabs(2);

    open_file(&mut h, "a.rs");
    h.run_cycles(1);
    open_file(&mut h, "b.rs");
    h.run_cycles(1);
    // Now at limit (2). Open c.rs — should evict LRU (a.rs, clean)
    open_file(&mut h, "c.rs");
    h.run_cycles(1);

    assert!(h.content_contains("ccc"));
}

#[test]
fn dirty_lru_triggers_close_prompt() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    h.state.settings_mut().set_max_tabs(2);
    h.state.settings_mut().editor_defaults_mut().set_autosave(false);

    open_file(&mut h, "a.rs");
    h.run_cycles(2);
    // Make a.rs dirty while focused
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    open_file(&mut h, "b.rs");
    h.run_cycles(2);
    // Now at limit. Open c.rs — dirty LRU triggers close prompt
    open_file(&mut h, "c.rs");
    h.run_cycles(2);

    // pending_tab should be set (waiting for close prompt resolution)
    assert!(h.state.pending_tab().is_some());
    // a.rs should be the active tab (activated for close prompt)
    assert!(h.content_contains("Xaaa"));
}

#[test]
fn cancel_drops_new_tab() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    h.state.settings_mut().set_max_tabs(2);
    h.state.settings_mut().editor_defaults_mut().set_autosave(false);

    open_file(&mut h, "a.rs");
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    open_file(&mut h, "b.rs");
    h.run_cycles(2);
    open_file(&mut h, "c.rs");
    h.run_cycles(2);

    // Press Esc to cancel the close prompt
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // c.rs should NOT be open
    assert!(!h.content_contains("ccc"));
    // a.rs content should still be visible (it's active)
    assert!(h.content_contains("Xaaa"));
}

#[test]
fn discard_evicts_and_opens_new() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    h.state.settings_mut().set_max_tabs(2);
    h.state.settings_mut().editor_defaults_mut().set_autosave(false);

    open_file(&mut h, "a.rs");
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    open_file(&mut h, "b.rs");
    h.run_cycles(2);
    open_file(&mut h, "c.rs");
    h.run_cycles(4);

    // Press 'n' to discard (ConfirmItem prompt: [y]es [n]o [Esc]cancel)
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(8);

    // c.rs should now be open
    assert!(h.content_contains("ccc"));
}
