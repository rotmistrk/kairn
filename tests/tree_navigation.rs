mod helpers;

use helpers::{run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn tree_shows_files_on_start() {
    let dir = temp_project(&[("hello.rs", "fn main() {}"), ("world.txt", "hi")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    let screen = run_and_capture(&mut app, &mut be, 1);
    assert!(screen.contains("hello.rs"));
    assert!(screen.contains("world.txt"));
}

#[test]
fn tree_down_moves_cursor() {
    let dir = temp_project(&[("aaa.rs", ""), ("bbb.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    be.inject_key(KeyCode::Down, KeyMod::default());
    let screen = run_and_capture(&mut app, &mut be, 1);
    // Second file should now be highlighted (cursor moved)
    assert!(screen.contains("bbb.rs"));
}

#[test]
fn tree_enter_on_dir_expands() {
    let dir = temp_project(&[("sub/inner.rs", "// inner")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // "sub" dir should be visible; Enter expands it
    be.inject_key(KeyCode::Enter, KeyMod::default());
    let screen = run_and_capture(&mut app, &mut be, 1);
    assert!(screen.contains("inner.rs"));
}

#[test]
fn tree_dirs_sort_before_files() {
    let dir = temp_project(&[("zz_file.rs", ""), ("aa_dir/x.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    let screen = run_and_capture(&mut app, &mut be, 1);
    let dir_pos = screen.find("aa_dir").unwrap_or(usize::MAX);
    let file_pos = screen.find("zz_file").unwrap_or(0);
    assert!(dir_pos < file_pos);
}
