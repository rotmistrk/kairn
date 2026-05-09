mod helpers;

use helpers::{setup, temp_project};
use txv_core::event::Event;
use txv_core::run::run_cycles;

#[test]
fn resize_recomputes_layout() {
    let dir = temp_project(&[("a.rs", "content")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_cycles(&mut app, &mut be, 1);
    // Resize to smaller
    be.inject(Event::Resize(60, 20));
    run_cycles(&mut app, &mut be, 1);
    assert!(be.contains("a.rs"));
}

#[test]
fn small_terminal_still_renders() {
    let dir = temp_project(&[("a.rs", "hi")]);
    let (mut app, mut be) = setup(dir.path(), 40, 10);
    run_cycles(&mut app, &mut be, 1);
    assert!(!be.screen_text().is_empty());
}
