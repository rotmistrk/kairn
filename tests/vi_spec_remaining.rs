//! Vi spec: remaining coverage, undo, movement, set options.
use kairn::editor::command::Command;
use kairn::editor::keymap::EditorMode;
use kairn::editor::{Editor, EditorAction};

// === Remaining spec coverage ===

#[test]
fn count_5k_moves_up_5() {
    let text = (0..10).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.cursor_line = 7;
    ed.execute(Command::Repeat(5, Box::new(Command::MoveUp)));
    assert_eq!(ed.cursor_line, 2);
}

#[test]
fn dw_deletes_word() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::DeleteWord);
    assert_eq!(ed.buffer.content(), "world");
}

#[test]
fn cw_changes_word() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::ChangeWord);
    assert_eq!(ed.mode, EditorMode::Insert);
    // "hello " deleted, cursor at start
    assert_eq!(ed.buffer.content(), "world");
}

#[test]
fn dedent_removes_spaces() {
    let mut ed = Editor::from_text("    hello");
    ed.execute(Command::Unindent);
    assert_eq!(ed.buffer.content(), "hello");
}

#[test]
fn count_3_dedent() {
    let mut ed = Editor::from_text("    a\n    b\n    c\nd");
    ed.execute(Command::Repeat(3, Box::new(Command::Unindent)));
    assert_eq!(ed.buffer.content(), "a\nb\nc\nd");
}

#[test]
fn insert_backspace_deletes_backward() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::EnterInsertMode);
    ed.execute(Command::MoveRight);
    ed.execute(Command::MoveRight);
    ed.execute(Command::DeleteCharBackward);
    assert_eq!(ed.buffer.content(), "hllo");
}

#[test]
fn ex_relative_range_delete() {
    let mut ed = Editor::from_text("a\nb\nc\nd\ne");
    ed.cursor_line = 1; // on "b"
    ed.execute(Command::ExCommand(".,+2d".to_string()));
    // Should delete lines 1,2,3 (b,c,d)
    assert_eq!(ed.buffer.content(), "a\ne");
}

#[test]
fn ex_relative_range_yank() {
    let mut ed = Editor::from_text("a\nb\nc\nd\ne");
    ed.cursor_line = 1;
    ed.execute(Command::ExCommand(".,+2y".to_string()));
    assert_eq!(ed.register, "b\nc\nd\n");
}

#[test]
fn search_n_finds_next() {
    let mut ed = Editor::from_text("foo bar foo baz foo");
    ed.execute(Command::SearchForward("foo".to_string()));
    assert_eq!(ed.cursor_col, 8); // second foo
    ed.execute(Command::SearchNext);
    assert_eq!(ed.cursor_col, 16); // third foo
}

#[test]
fn search_big_n_finds_prev() {
    let mut ed = Editor::from_text("foo bar foo baz foo");
    ed.execute(Command::SearchForward("foo".to_string()));
    ed.execute(Command::SearchNext); // at third foo (col 16)
    ed.execute(Command::SearchPrev);
    assert_eq!(ed.cursor_col, 8); // back to second foo
}

#[test]
fn esc_exits_visual() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::EnterVisual);
    assert_eq!(ed.mode, EditorMode::Visual);
    ed.execute(Command::ExitVisual);
    assert_eq!(ed.mode, EditorMode::Normal);
}

#[test]
fn visual_line_yank() {
    let mut ed = Editor::from_text("line1\nline2\nline3");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualYank);
    assert_eq!(ed.register, "line1\nline2\n");
    assert_eq!(ed.mode, EditorMode::Normal);
}

#[test]
fn visual_indent_selection() {
    let mut ed = Editor::from_text("a\nb\nc");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualIndent);
    assert!(ed.buffer.content().starts_with("    a\n    b\nc"));
}

#[test]
fn visual_unindent_selection() {
    let mut ed = Editor::from_text("    a\n    b\nc");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualUnindent);
    assert!(ed.buffer.content().starts_with("a\nb\nc"));
}

#[test]
fn ex_wq_saves() {
    let mut ed = Editor::from_text("hello");
    let action = ed.execute(Command::ExCommand("wq".to_string()));
    assert_eq!(action, EditorAction::SaveRequested);
}

#[test]
fn undo_redo_cycle() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::DeleteCharForward);
    assert_eq!(ed.buffer.content(), "ello");
    ed.execute(Command::Undo);
    assert_eq!(ed.buffer.content(), "hello");
    ed.execute(Command::Redo);
    assert_eq!(ed.buffer.content(), "ello");
}

#[test]
fn gg_with_count_goes_to_line() {
    let text = (1..=20).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.execute(Command::GotoLine(5));
    assert_eq!(ed.cursor_line, 4); // 0-indexed
}

// === :set list/nolist and :set number/nonumber ===

#[test]
fn set_list_enables_list_mode() {
    let mut ed = Editor::from_text("hello");
    assert!(!ed.options.list);
    ed.execute(Command::ExCommand("set list".to_string()));
    assert!(ed.options.list);
}

#[test]
fn set_nolist_disables_list_mode() {
    let mut ed = Editor::from_text("hello");
    ed.options.list = true;
    ed.execute(Command::ExCommand("set nolist".to_string()));
    assert!(!ed.options.list);
}

#[test]
fn set_li_abbreviation() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::ExCommand("set li".to_string()));
    assert!(ed.options.list);
}

#[test]
fn set_noli_abbreviation() {
    let mut ed = Editor::from_text("hello");
    ed.options.list = true;
    ed.execute(Command::ExCommand("set noli".to_string()));
    assert!(!ed.options.list);
}

#[test]
fn set_number_enables_line_numbers() {
    let mut ed = Editor::from_text("hello");
    ed.options.number = false;
    ed.execute(Command::ExCommand("set number".to_string()));
    assert!(ed.options.number);
}

#[test]
fn set_nonumber_disables_line_numbers() {
    let mut ed = Editor::from_text("hello");
    assert!(ed.options.number); // default on
    ed.execute(Command::ExCommand("set nonumber".to_string()));
    assert!(!ed.options.number);
}

#[test]
fn set_nu_abbreviation() {
    let mut ed = Editor::from_text("hello");
    ed.options.number = false;
    ed.execute(Command::ExCommand("set nu".to_string()));
    assert!(ed.options.number);
}

#[test]
fn set_nonu_abbreviation() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::ExCommand("set nonu".to_string()));
    assert!(!ed.options.number);
}
