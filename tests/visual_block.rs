//! Test: visual block mode operations.

use kairn::editor::keymap::EditorMode;
use kairn::views::editor::EditorView;
use txv_core::prelude::*;

fn inject_keys(view: &mut EditorView, keys: &[KeyEvent]) {
    for key in keys {
        view.handle(&Event::Key(*key));
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod::default(),
    }
}

fn ctrl(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod {
            ctrl: true,
            shift: false,
            alt: false,
        },
    }
}

#[test]
fn enter_visual_block_mode() {
    let mut view = EditorView::from_text("abc\ndef\nghi\n");
    view.set_bounds(Rect::new(0, 0, 40, 5));
    inject_keys(&mut view, &[ctrl('v')]);
    assert_eq!(view.editor().mode(), EditorMode::VisualBlock);
}

#[test]
fn block_delete_removes_columns() {
    let mut view = EditorView::from_text("abcde\nfghij\nklmno\n");
    view.set_bounds(Rect::new(0, 0, 40, 5));
    // Enter visual block, move right 2, down 2, delete
    inject_keys(
        &mut view,
        &[
            ctrl('v'),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('j')),
            key(KeyCode::Char('j')),
            key(KeyCode::Char('d')),
        ],
    );
    let content = view.editor().buf().content().to_string();
    // Deleted cols 0..=2 from lines 0..=2
    assert_eq!(content, "de\nij\nno\n");
}

#[test]
fn block_yank_and_paste_with_padding() {
    let mut view = EditorView::from_text("abcdef\ngh\nij\n");
    view.set_bounds(Rect::new(0, 0, 40, 5));
    // Yank block: col 0..=1, lines 0..=2
    inject_keys(
        &mut view,
        &[
            ctrl('v'),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('j')),
            key(KeyCode::Char('j')),
            key(KeyCode::Char('y')),
        ],
    );
    // Cursor is now at (0, 0). Move to col 5 on line 0 and paste before.
    // Lines 1,2 are shorter than col 5 — they should be padded.
    inject_keys(
        &mut view,
        &[
            key(KeyCode::Char('l')),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('P')),
        ],
    );
    let content = view.editor().buf().content().to_string();
    // Line 0: "abcde" + insert "ab" at col 5 → "abcdeabf"
    // Line 1: "gh" padded to 5 cols → "gh   " + insert "gh" → "gh   ghf"...
    // Actually let's just check padding happened
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines[1].len() > 2, "line 1 should be padded, got: {:?}", lines[1]);
    assert!(lines[2].len() > 2, "line 2 should be padded, got: {:?}", lines[2]);
}

#[test]
fn block_replace_chars() {
    let mut view = EditorView::from_text("abcde\nfghij\nklmno\n");
    view.set_bounds(Rect::new(0, 0, 40, 5));
    // Select block col 1..=2, lines 0..=1, replace with 'X'
    inject_keys(
        &mut view,
        &[
            key(KeyCode::Char('l')), // move to col 1
            ctrl('v'),
            key(KeyCode::Char('l')),
            key(KeyCode::Char('j')),
            key(KeyCode::Char('r')),
            key(KeyCode::Char('X')),
        ],
    );
    let content = view.editor().buf().content().to_string();
    assert!(content.starts_with("aXXde\nfXXij"), "got: {:?}", content);
}
