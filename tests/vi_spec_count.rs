//! Vi spec: count prefix, operators, visual mode, ex commands.
use kairn::editor::command::Command;
use kairn::editor::keymap::EditorMode;
use kairn::editor::{Editor, EditorAction};

// === Count prefix tests ===

#[test]
fn count_5j_moves_down_5() {
    let text = (0..10).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.execute(Command::MoveDown); // verify single works
    assert_eq!(ed.cursor_line(), 1);
    ed.set_cursor_line(0);
    ed.execute(Command::Repeat(5, Box::new(Command::MoveDown)));
    assert_eq!(ed.cursor_line(), 5);
}

#[test]
fn count_3w_moves_3_words() {
    let mut ed = Editor::from_text("one two three four five");
    ed.execute(Command::Repeat(3, Box::new(Command::MoveWordForward)));
    // Should be at "four" (word 3, 0-indexed positions: one=0, two=4, three=8, four=14)
    assert_eq!(ed.cursor_col(), 14);
}

#[test]
fn count_5x_deletes_5_chars() {
    let mut ed = Editor::from_text("abcdefgh");
    ed.execute(Command::Repeat(5, Box::new(Command::DeleteCharForward)));
    assert_eq!(ed.buf().content(), "fgh");
}

#[test]
fn count_3dd_deletes_3_lines() {
    let mut ed = Editor::from_text("line1\nline2\nline3\nline4\nline5");
    ed.execute(Command::Repeat(3, Box::new(Command::DeleteLine)));
    assert_eq!(ed.buf().content(), "line4\nline5");
}

#[test]
fn count_5yy_yanks_5_lines() {
    let text = "a\nb\nc\nd\ne\nf";
    let mut ed = Editor::from_text(text);
    ed.execute(Command::Repeat(5, Box::new(Command::YankLine)));
    // Repeat(5, YankLine) yanks 5 lines (multi-line yank)
    assert_eq!(ed.register(), "a\nb\nc\nd\ne\n");
}

#[test]
fn count_3cc_changes_3_lines() {
    let mut ed = Editor::from_text("line1\nline2\nline3\nline4");
    ed.execute(Command::Repeat(3, Box::new(Command::ChangeLine)));
    // 3 lines deleted, in insert mode
    assert_eq!(ed.mode(), EditorMode::Insert);
    assert_eq!(ed.buf().content(), "\nline4");
}

#[test]
fn count_3_indent() {
    let mut ed = Editor::from_text("a\nb\nc\nd");
    ed.execute(Command::Repeat(3, Box::new(Command::Indent)));
    let content = ed.buf().content();
    assert!(content.starts_with("    a\n    b\n    c\nd"), "got: {:?}", content);
}

#[test]
fn count_10g_goto_line_10() {
    let text = (1..=20).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.execute(Command::GotoLine(10));
    assert_eq!(ed.cursor_line(), 9); // 0-indexed
}

// === Operator motion tests ===

#[test]
fn db_deletes_word_backward() {
    let mut ed = Editor::from_text("hello world");
    ed.set_cursor_col(6); // at 'w' of "world"
    ed.execute(Command::DeleteWordBackward);
    // db from 'w' goes back to start of previous word "hello" → deletes "hello "
    assert_eq!(ed.buf().content(), "world");
    assert_eq!(ed.cursor_col(), 0);
}

#[test]
fn d0_deletes_to_line_start() {
    let mut ed = Editor::from_text("hello world");
    ed.set_cursor_col(6);
    ed.execute(Command::DeleteToStart);
    assert_eq!(ed.buf().content(), "world");
    assert_eq!(ed.cursor_col(), 0);
}

#[test]
fn big_d_deletes_to_end() {
    let mut ed = Editor::from_text("hello world");
    ed.set_cursor_col(5);
    ed.execute(Command::DeleteToEnd);
    assert_eq!(ed.buf().content(), "hello");
}

#[test]
fn big_c_changes_to_end() {
    let mut ed = Editor::from_text("hello world");
    ed.set_cursor_col(5);
    ed.execute(Command::ChangeToEnd);
    assert_eq!(ed.buf().content(), "hello");
    assert_eq!(ed.mode(), EditorMode::Insert);
}

#[test]
fn yw_yanks_word() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::YankWord);
    assert_eq!(ed.register(), "hello ");
}

#[test]
fn y_dollar_yanks_to_end() {
    let mut ed = Editor::from_text("hello world");
    ed.set_cursor_col(6);
    ed.execute(Command::YankToEnd);
    assert_eq!(ed.register(), "world");
}

// === Visual mode c and : ===

#[test]
fn visual_c_changes_selection() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::EnterVisual);
    for _ in 0..4 {
        ed.execute(Command::MoveRight);
    }
    ed.execute(Command::VisualChange);
    assert_eq!(ed.mode(), EditorMode::Insert);
    assert_eq!(ed.buf().content(), " world");
}

#[test]
fn visual_colon_sets_range() {
    let mut ed = Editor::from_text("line1\nline2\nline3\nline4");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualExCommand);
    assert_eq!(ed.mode(), EditorMode::Command);
}

// === Ex command gaps ===

#[test]
fn ex_q_fails_on_dirty_buffer() {
    let mut ed = Editor::from_text("hello");
    // Enter insert mode and type to make buffer dirty
    ed.execute(Command::EnterInsertMode);
    ed.execute(Command::InsertChar('x'));
    ed.execute(Command::ExitInsertMode);
    let action = ed.execute(Command::ExCommand("q".to_string()));
    // Should NOT close — buffer is dirty
    assert_ne!(action, EditorAction::CloseRequested);
    assert!(
        ed.status().contains("write") || ed.status().contains("modified") || ed.status().contains("unsaved"),
        "expected dirty warning, got: {:?}",
        ed.status()
    );
}

#[test]
fn ex_q_bang_force_closes() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::InsertChar('x')); // make dirty
    ed.set_mode(EditorMode::Normal);
    let action = ed.execute(Command::ExCommand("q!".to_string()));
    assert_eq!(action, EditorAction::ForceCloseRequested);
}

#[test]
fn ex_e_opens_file() {
    let mut ed = Editor::from_text("hello");
    let action = ed.execute(Command::ExCommand("e somefile.rs".to_string()));
    // Should signal that a file needs to be opened
    assert_eq!(action, EditorAction::OpenFile("somefile.rs".to_string()));
}

#[test]
fn ex_bang_command_bare() {
    let mut ed = Editor::from_text("hello");
    let action = ed.execute(Command::ExCommand("!echo hi".to_string()));
    // Should run command and show output (not filter current line)
    assert_eq!(action, EditorAction::ShellOutput("hi\n".to_string()));
}
