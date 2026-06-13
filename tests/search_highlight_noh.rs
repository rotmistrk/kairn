//! Test: `:noh` clears search highlight end-to-end.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

fn open_and_focus(h: &mut TestHarness, path: &std::path::Path) {
    let req = OpenFileRequest::new(path.to_path_buf());
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);
}

#[test]
fn noh_clears_search_highlight() {
    let dir = temp_project(&[("t.txt", "hello world hello")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, &dir.path().join("t.txt"));

    // Search for /hello
    h.inject_key(KeyCode::Char('/'), KeyMod::default());
    h.inject_str("hello");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    // Verify search worked (content is visible, cursor moved to match)
    assert!(h.content_contains("hello world hello"));

    // Verify highlight is active by checking rendered cells have highlight bg
    let buf = h.backend.buffer().unwrap();
    let has_highlight_bg = find_highlight_cells(buf);
    assert!(has_highlight_bg, "search should produce highlighted cells");

    // Send :noh
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("noh");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    // Verify highlight is cleared
    let buf = h.backend.buffer().unwrap();
    let has_highlight_bg = find_highlight_cells(buf);
    assert!(!has_highlight_bg, "highlight should be cleared after :noh");

    // Editor still functional (back to NOR mode)
    assert!(h.contains("NOR"));
}

/// Check if any cell in the content area has the search highlight background.
fn find_highlight_cells(buf: &txv_core::buffer::Buffer) -> bool {
    use txv_core::cell::Color;
    let w = buf.width();
    let h = buf.height();
    let match_bg = Color::Rgb(0x44, 0x44, 0x00);
    let other_bg = Color::Rgb(0x00, 0x44, 0x00);
    for y in 1..h.saturating_sub(1) {
        for x in 0..w {
            let bg = buf.cell(x, y).style().bg();
            if bg == match_bg || bg == other_bg {
                return true;
            }
        }
    }
    false
}

/// Check that search highlight cells contain the character 'a' with non-default bg.
fn has_highlighted_char(buf: &txv_core::buffer::Buffer, ch: char) -> bool {
    use txv_core::cell::Color;
    let w = buf.width();
    let h = buf.height();
    for y in 1..h.saturating_sub(1) {
        for x in 0..w {
            let cell = buf.cell(x, y);
            if cell.ch() == ch && cell.style().bg() != Color::Reset {
                return true;
            }
        }
    }
    false
}

#[test]
fn search_highlight_visible_on_cells() {
    let dir = temp_project(&[("f.txt", "aaa bbb aaa ccc aaa")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, &dir.path().join("f.txt"));

    // Search for /aaa
    h.inject_key(KeyCode::Char('/'), KeyMod::default());
    h.inject_str("aaa");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    let buf = h.backend.buffer().unwrap();
    assert!(
        has_highlighted_char(buf, 'a'),
        "at least one 'a' cell should have non-default background (search highlight)"
    );
}
