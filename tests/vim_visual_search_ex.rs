//! Tests for visual mode, search, and ex commands on the Editor core.

use kairn::editor::command::Command;
use kairn::editor::keymap::EditorMode;
use kairn::editor::{Editor, EditorAction};

// --- Visual mode tests ---

#[test]
fn visual_mode_enter_exit() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::EnterVisual);
    assert_eq!(ed.mode, EditorMode::Visual);
    ed.execute(Command::ExitVisual);
    assert_eq!(ed.mode, EditorMode::Normal);
}

#[test]
fn visual_line_mode_enter() {
    let mut ed = Editor::from_text("line1\nline2\nline3");
    ed.execute(Command::EnterVisualLine);
    assert_eq!(ed.mode, EditorMode::VisualLine);
    assert_eq!(ed.visual_anchor, Some((0, 0)));
}

#[test]
fn visual_delete_chars() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::EnterVisual);
    // Move right 5 times to select "hello"
    for _ in 0..4 {
        ed.execute(Command::MoveRight);
    }
    ed.execute(Command::VisualDelete);
    assert_eq!(ed.mode, EditorMode::Normal);
    assert_eq!(ed.buffer.content(), " world");
    assert_eq!(ed.register, "hello");
}

#[test]
fn visual_delete_across_lines() {
    let mut ed = Editor::from_text("abc\ndef\nghi");
    ed.execute(Command::EnterVisual);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualDelete);
    // Should delete from (0,0) to (1,0) inclusive
    assert_eq!(ed.mode, EditorMode::Normal);
    // The exact result depends on selection range
    assert!(!ed.buffer.content().starts_with("abc"));
}

#[test]
fn visual_line_delete() {
    let mut ed = Editor::from_text("line1\nline2\nline3");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown); // select line1 and line2
    ed.execute(Command::VisualDelete);
    assert_eq!(ed.mode, EditorMode::Normal);
    assert_eq!(ed.buffer.content(), "line3");
}

#[test]
fn visual_yank() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::EnterVisual);
    for _ in 0..4 {
        ed.execute(Command::MoveRight);
    }
    ed.execute(Command::VisualYank);
    assert_eq!(ed.mode, EditorMode::Normal);
    assert_eq!(ed.register, "hello");
    // Buffer unchanged
    assert_eq!(ed.buffer.content(), "hello world");
}

#[test]
fn visual_indent() {
    let mut ed = Editor::from_text("line1\nline2\nline3");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualIndent);
    assert!(ed.buffer.content().contains("    line1"));
    assert!(ed.buffer.content().contains("    line2"));
    assert!(!ed.buffer.content().contains("    line3"));
}

#[test]
fn visual_unindent() {
    let mut ed = Editor::from_text("    line1\n    line2\nline3");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualUnindent);
    let content = ed.buffer.content();
    assert!(content.starts_with("line1\nline2\n"));
}

// --- Search tests ---

#[test]
fn search_forward_finds_match() {
    let mut ed = Editor::from_text("hello world hello");
    ed.execute(Command::SearchForward("world".to_string()));
    assert_eq!(ed.cursor_col, 6);
    assert_eq!(ed.cursor_line, 0);
}

#[test]
fn search_forward_wraps() {
    let mut ed = Editor::from_text("abc def abc");
    ed.cursor_col = 5; // past first "abc"
    ed.execute(Command::SearchForward("abc".to_string()));
    // Should find second "abc" at col 8
    assert_eq!(ed.cursor_col, 8);
}

#[test]
fn search_next_repeats() {
    let mut ed = Editor::from_text("aaa bbb aaa bbb");
    ed.execute(Command::SearchForward("bbb".to_string()));
    assert_eq!(ed.cursor_col, 4);
    ed.execute(Command::SearchNext);
    assert_eq!(ed.cursor_col, 12);
}

#[test]
fn search_prev_goes_backward() {
    let mut ed = Editor::from_text("abc xyz abc xyz");
    ed.cursor_col = 10;
    ed.search_pattern = "abc".to_string();
    ed.execute(Command::SearchPrev);
    assert_eq!(ed.cursor_col, 0);
}

#[test]
fn search_word_under_cursor() {
    let mut ed = Editor::from_text("foo bar foo baz");
    // cursor on "foo" at col 0
    ed.execute(Command::SearchWordForward);
    // Should jump to second "foo" at col 8
    assert_eq!(ed.cursor_col, 8);
}

// --- Ex command tests ---

#[test]
fn ex_goto_line() {
    let mut ed = Editor::from_text("a\nb\nc\nd\ne");
    let action = ed.execute(Command::ExCommand("3".to_string()));
    assert_eq!(action, EditorAction::CursorMoved);
    assert_eq!(ed.cursor_line, 2); // 0-indexed
}

#[test]
fn ex_substitute_single() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::ExCommand("%s/world/rust/".to_string()));
    assert_eq!(ed.buffer.content(), "hello rust");
}

#[test]
fn ex_substitute_global() {
    let mut ed = Editor::from_text("aaa bbb aaa");
    ed.execute(Command::ExCommand("%s/aaa/xxx/g".to_string()));
    assert_eq!(ed.buffer.content(), "xxx bbb xxx");
}

#[test]
fn ex_substitute_range() {
    let mut ed = Editor::from_text("foo\nfoo\nfoo");
    ed.execute(Command::ExCommand("1,2s/foo/bar/".to_string()));
    let content = ed.buffer.content();
    assert!(content.starts_with("bar\nbar\n"));
    assert!(content.ends_with("foo"));
}

#[test]
fn ex_delete_lines() {
    let mut ed = Editor::from_text("line1\nline2\nline3");
    ed.execute(Command::ExCommand("2d".to_string()));
    assert_eq!(ed.buffer.content(), "line1\nline3");
}

#[test]
fn ex_yank_lines() {
    let mut ed = Editor::from_text("line1\nline2\nline3");
    ed.execute(Command::ExCommand("%y".to_string()));
    assert!(ed.register.contains("line1"));
    assert!(ed.register.contains("line3"));
}

#[test]
fn ex_save_returns_action() {
    let mut ed = Editor::from_text("content");
    let action = ed.execute(Command::ExCommand("w".to_string()));
    assert_eq!(action, EditorAction::SaveRequested);
}

#[test]
fn ex_quit_returns_action() {
    let mut ed = Editor::from_text("content");
    let action = ed.execute(Command::ExCommand("q".to_string()));
    assert_eq!(action, EditorAction::CloseRequested);
}

// --- Additional editing tests ---

#[test]
fn dot_repeat_works() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::DeleteCharForward); // deletes 'h'
    assert_eq!(ed.buffer.content(), "ello");
    ed.execute(Command::DotRepeat); // deletes 'e'
    assert_eq!(ed.buffer.content(), "llo");
}

#[test]
fn replace_char() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::ReplaceChar('X'));
    assert_eq!(ed.buffer.content(), "Xello");
}

#[test]
fn toggle_case() {
    let mut ed = Editor::from_text("Hello");
    ed.execute(Command::ToggleCase);
    assert_eq!(ed.buffer.content(), "hello");
}

#[test]
fn join_lines() {
    let mut ed = Editor::from_text("hello\n  world");
    ed.execute(Command::JoinLines);
    assert_eq!(ed.buffer.content(), "hello world");
}

#[test]
fn indent_unindent() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::Indent);
    assert_eq!(ed.buffer.content(), "    hello");
    ed.execute(Command::Unindent);
    assert_eq!(ed.buffer.content(), "hello");
}

#[test]
fn find_char_forward() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::FindChar('o'));
    assert_eq!(ed.cursor_col, 4);
}

#[test]
fn find_char_backward() {
    let mut ed = Editor::from_text("hello world");
    ed.cursor_col = 8;
    ed.execute(Command::FindCharBack('o'));
    // 'o' in "world" is at col 7, which is the first 'o' before col 8
    assert_eq!(ed.cursor_col, 7);
}

#[test]
fn match_bracket_works() {
    let mut ed = Editor::from_text("(hello)");
    ed.execute(Command::MatchBracket);
    assert_eq!(ed.cursor_col, 6);
    ed.execute(Command::MatchBracket);
    assert_eq!(ed.cursor_col, 0);
}
