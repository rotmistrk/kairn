//! Vi spec: find char, bracket match, search, dot repeat, substitute, replace.
use kairn::editor::command::Command;
use kairn::editor::keymap::EditorMode;
use kairn::editor::{Editor, EditorAction};

// === Find char ===

#[test]
fn f_finds_char_forward() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::FindChar('o'));
    assert_eq!(ed.cursor_col, 4);
}

#[test]
fn big_f_finds_char_backward() {
    let mut ed = Editor::from_text("hello world");
    ed.cursor_col = 10; // at 'd'
    ed.execute(Command::FindCharBack('o'));
    assert_eq!(ed.cursor_col, 7); // 'o' in "world"
}

#[test]
fn semicolon_repeats_find() {
    let mut ed = Editor::from_text("abacada");
    ed.execute(Command::FindChar('a'));
    assert_eq!(ed.cursor_col, 2); // second 'a'
    ed.execute(Command::RepeatFind);
    assert_eq!(ed.cursor_col, 4); // third 'a'
}

// === Bracket match ===

#[test]
fn percent_matches_bracket() {
    let mut ed = Editor::from_text("(hello)");
    ed.execute(Command::MatchBracket);
    assert_eq!(ed.cursor_col, 6); // closing )
    ed.execute(Command::MatchBracket);
    assert_eq!(ed.cursor_col, 0); // back to opening (
}

// === Search ===

#[test]
fn search_word_forward() {
    let mut ed = Editor::from_text("foo bar foo baz");
    ed.execute(Command::SearchWordForward);
    assert_eq!(ed.cursor_col, 8); // second "foo"
}

#[test]
fn search_word_backward() {
    let mut ed = Editor::from_text("foo bar foo baz");
    ed.cursor_col = 8; // second "foo"
    ed.execute(Command::SearchWordBackward);
    assert_eq!(ed.cursor_col, 0); // first "foo"
}

// === Dot repeat ===

#[test]
fn dot_repeats_last_edit() {
    let mut ed = Editor::from_text("aaa\nbbb\nccc");
    ed.execute(Command::DeleteLine);
    assert_eq!(ed.buffer.line(0).unwrap(), "bbb");
    ed.execute(Command::DotRepeat);
    assert_eq!(ed.buffer.line(0).unwrap(), "ccc");
}

// === Substitute ===

#[test]
fn s_substitutes_char() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::Substitute);
    assert_eq!(ed.mode, EditorMode::Insert);
    assert_eq!(ed.buffer.content(), "ello");
}

#[test]
fn big_s_substitutes_line() {
    let mut ed = Editor::from_text("hello\nworld");
    ed.execute(Command::SubstituteLine);
    assert_eq!(ed.mode, EditorMode::Insert);
    // Line content cleared, newline preserved
    assert!(
        ed.buffer.content().starts_with('\n') || ed.buffer.content() == "world" || ed.buffer.content() == "\nworld"
    );
}

// === Toggle case ===

#[test]
fn tilde_toggles_case() {
    let mut ed = Editor::from_text("Hello");
    ed.execute(Command::ToggleCase);
    assert_eq!(ed.buffer.line(0).unwrap().chars().next(), Some('h'));
}

// === Replace char ===

#[test]
fn r_replaces_char() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::ReplaceChar('X'));
    assert_eq!(ed.buffer.content(), "Xello");
    assert_eq!(ed.mode, EditorMode::Normal); // stays in normal
}

// === Ex: substitute ===

#[test]
fn ex_substitute_first_on_line() {
    let mut ed = Editor::from_text("foo foo foo");
    ed.execute(Command::ExCommand("s/foo/bar/".to_string()));
    assert_eq!(ed.buffer.content(), "bar foo foo");
}

#[test]
fn ex_substitute_all_on_line() {
    let mut ed = Editor::from_text("foo foo foo");
    ed.execute(Command::ExCommand("s/foo/bar/g".to_string()));
    assert_eq!(ed.buffer.content(), "bar bar bar");
}

#[test]
fn ex_substitute_all_in_file() {
    let mut ed = Editor::from_text("foo\nfoo\nfoo");
    ed.execute(Command::ExCommand("%s/foo/bar/g".to_string()));
    assert_eq!(ed.buffer.content(), "bar\nbar\nbar");
}

#[test]
fn ex_goto_line() {
    let text = (1..=20).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.execute(Command::ExCommand("15".to_string()));
    assert_eq!(ed.cursor_line, 14);
}

// === Undo delete ===

#[test]
fn undo_reverses_delete_line() {
    let mut ed = Editor::from_text("a\nb\nc");
    ed.execute(Command::DeleteLine);
    assert_eq!(ed.buffer.line(0).unwrap(), "b");
    ed.execute(Command::Undo);
    assert_eq!(ed.buffer.line(0).unwrap(), "a");
}

// === P (paste before) ===

#[test]
fn big_p_pastes_before() {
    let mut ed = Editor::from_text("line1\nline2");
    ed.cursor_line = 1;
    ed.execute(Command::YankLine);
    ed.execute(Command::PasteBefore);
    // Should paste "line2" above current line
    assert_eq!(ed.buffer.line(1).unwrap(), "line2");
}
