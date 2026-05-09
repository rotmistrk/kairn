mod helpers;

use helpers::{run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn enter_opens_file_in_center_slot() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Enter on file opens it
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Tab bar should show filename
    assert!(be.row(0).contains("main.rs"));
}
#[test]
fn open_same_file_twice_focuses_existing() {
    let dir = temp_project(&[("only.rs", "hello")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Focus back to tree (F2)
    be.inject_key(KeyCode::F(2), KeyMod::default());
    // Try to open same file again
    be.inject_key(KeyCode::Enter, KeyMod::default());
    let screen = run_and_capture(&mut app, &mut be, 1);
    // Should still only have one tab with that name
    let count = screen.matches("only.rs").count();
    // Tab bar + tree = 2 occurrences max (one in tree, one in tab bar)
    assert!(count <= 2, "file opened twice: found {} occurrences", count);
}

#[test]
fn center_slot_shows_file_content() {
    let dir = temp_project(&[("hello.rs", "fn greet() { println!(\"hi\"); }")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    be.inject_key(KeyCode::Enter, KeyMod::default());
    let screen = run_and_capture(&mut app, &mut be, 1);
    assert!(screen.contains("fn greet()"));
}

#[test]
fn multiple_files_create_multiple_tabs() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Open first file
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Back to tree, move down, open second
    be.inject_key(KeyCode::F(2), KeyMod::default());
    be.inject_key(KeyCode::Down, KeyMod::default());
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    let tab_bar = be.row(0);
    assert!(tab_bar.contains("a.rs"));
    assert!(tab_bar.contains("b.rs"));
}
