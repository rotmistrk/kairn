//! Vi spec: paste, insert mode, keymap integration, entry variants.
use kairn::editor::command::Command;
use kairn::editor::keymap::EditorMode;
use kairn::editor::{Editor, EditorAction};

// === Paste undo ===

#[test]
fn paste_is_undoable() {
    let mut ed = Editor::from_text("line1\nline2");
    ed.execute(Command::YankLine);
    ed.execute(Command::Paste);
    assert!(ed.buf().content().contains("line1\nline1"));
    ed.execute(Command::Undo);
    assert_eq!(ed.buf().content(), "line1\nline2");
}

// === Insert mode features ===

#[test]
fn insert_enter_creates_newline() {
    let mut ed = Editor::from_text("hello");
    ed.set_mode(EditorMode::Insert);
    ed.set_cursor_col(3);
    ed.execute(Command::InsertNewline);
    assert_eq!(ed.buf().content(), "hel\nlo");
}

#[test]
fn insert_delete_forward() {
    let mut ed = Editor::from_text("hello");
    ed.set_mode(EditorMode::Insert);
    ed.execute(Command::DeleteCharForward);
    assert_eq!(ed.buf().content(), "ello");
}

// === 3J join 3 lines ===

#[test]
fn count_3j_joins_3_lines() {
    let mut ed = Editor::from_text("a\nb\nc\nd");
    ed.execute(Command::Repeat(3, Box::new(Command::JoinLines)));
    assert_eq!(ed.buf().content(), "a b c d");
}

// === Keymap integration: count prefix through keymap ===

use kairn::editor::keymap::Keymap;
use kairn::editor::keymap_vim::VimKeymap;
use txv_core::event::{KeyCode, KeyEvent, KeyMod};

fn key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyMod::default())
}

#[test]
fn keymap_5j_produces_repeat() {
    let mut km = VimKeymap::new();
    let cmd = km.handle_key(&key('5'), EditorMode::Normal);
    assert_eq!(cmd, Command::Noop); // accumulating
    let cmd = km.handle_key(&key('j'), EditorMode::Normal);
    assert_eq!(cmd, Command::Repeat(5, Box::new(Command::MoveDown)));
}

#[test]
fn keymap_3dd_produces_repeat() {
    let mut km = VimKeymap::new();
    km.handle_key(&key('3'), EditorMode::Normal);
    km.handle_key(&key('d'), EditorMode::Normal); // pending 'd'
    let cmd = km.handle_key(&key('d'), EditorMode::Normal);
    assert_eq!(cmd, Command::Repeat(3, Box::new(Command::DeleteLine)));
}

#[test]
fn keymap_10g_produces_goto_line() {
    let mut km = VimKeymap::new();
    km.handle_key(&key('1'), EditorMode::Normal);
    km.handle_key(&key('0'), EditorMode::Normal);
    let cmd = km.handle_key(&key('G'), EditorMode::Normal);
    assert_eq!(cmd, Command::GotoLine(10));
}

#[test]
fn keymap_no_count_g_is_file_end() {
    let mut km = VimKeymap::new();
    let cmd = km.handle_key(&key('G'), EditorMode::Normal);
    assert_eq!(cmd, Command::MoveFileEnd);
}

// === Insert mode entry variants ===

#[test]
fn a_inserts_after_cursor() {
    let mut ed = Editor::from_text("abc");
    ed.set_cursor_col(1); // on 'b'
    ed.execute(Command::EnterInsertAfter);
    ed.execute(Command::InsertChar('X'));
    assert_eq!(ed.buf().content(), "abXc");
}

#[test]
fn big_i_inserts_at_line_start() {
    let mut ed = Editor::from_text("  hello");
    ed.set_cursor_col(4);
    ed.execute(Command::EnterInsertLineStart);
    ed.execute(Command::InsertChar('X'));
    // I inserts before first non-blank (col 2)
    assert_eq!(ed.buf().content(), "  Xhello");
}

#[test]
fn big_a_inserts_at_line_end() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::EnterInsertLineEnd);
    ed.execute(Command::InsertChar('!'));
    assert_eq!(ed.buf().content(), "hello!");
}

#[test]
fn o_opens_line_below() {
    let mut ed = Editor::from_text("line1\nline2");
    ed.execute(Command::EnterInsertBelow);
    ed.execute(Command::InsertChar('X'));
    assert_eq!(ed.mode(), EditorMode::Insert);
    assert!(ed.buf().content().contains("line1\nX\nline2"));
}

#[test]
fn big_o_opens_line_above() {
    let mut ed = Editor::from_text("line1\nline2");
    ed.set_cursor_line(1);
    ed.execute(Command::EnterInsertAbove);
    ed.execute(Command::InsertChar('X'));
    assert_eq!(ed.mode(), EditorMode::Insert);
    assert!(ed.buf().content().contains("line1\nX\nline2"));
}

#[test]
fn insert_arrow_keys_move() {
    let mut ed = Editor::from_text("abc\ndef");
    ed.execute(Command::EnterInsertMode);
    ed.execute(Command::MoveRight);
    ed.execute(Command::MoveRight);
    ed.execute(Command::MoveDown);
    assert_eq!(ed.cursor_line(), 1);
    assert_eq!(ed.cursor_col(), 2);
}

// === Movement: e, ^, Ctrl-D/U, PgUp/PgDn ===

#[test]
fn e_moves_to_word_end() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::MoveWordEnd);
    assert_eq!(ed.cursor_col(), 4); // end of "hello"
}

#[test]
fn caret_moves_to_first_nonblank() {
    let mut ed = Editor::from_text("   hello");
    ed.set_cursor_col(6);
    ed.execute(Command::MoveFirstNonBlank);
    assert_eq!(ed.cursor_col(), 3);
}

#[test]
fn ctrl_d_moves_half_page_down() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.set_viewport_height(20);
    ed.execute(Command::HalfPageDown);
    assert_eq!(ed.cursor_line(), 10); // half of 20
}

#[test]
fn ctrl_u_moves_half_page_up() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.set_viewport_height(20);
    ed.set_cursor_line(20);
    ed.execute(Command::HalfPageUp);
    assert_eq!(ed.cursor_line(), 10);
}

#[test]
fn page_down_moves_full_page() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.set_viewport_height(20);
    ed.execute(Command::PageDown);
    // vim keeps 2 context lines, so moves viewport_height - 2
    assert_eq!(ed.cursor_line(), 18);
}

#[test]
fn page_up_moves_full_page() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.set_viewport_height(20);
    ed.set_cursor_line(30);
    ed.execute(Command::PageUp);
    assert_eq!(ed.cursor_line(), 12); // 30 - 18
}
