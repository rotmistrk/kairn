mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::Event;

#[test]
fn resize_recomputes_layout() {
    let dir = temp_project(&[("a.rs", "content")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.backend.inject(Event::Resize(60, 20));
    h.run_cycles(1);
    assert!(h.contains("a.rs"));
}

#[test]
fn small_terminal_still_renders() {
    let dir = temp_project(&[("a.rs", "hi")]);
    let mut h = TestHarness::with_size(dir.path(), 40, 10);
    h.run_cycles(1);
    assert!(!h.screen_text().is_empty());
}
