//! Tests for vi-command-spec features: count prefix, operators, visual c/:, ex gaps.

use kairn::editor::command::Command;
use kairn::editor::keymap::EditorMode;
use kairn::editor::{Editor, EditorAction};

// === Count prefix tests ===

#[test]
fn count_5j_moves_down_5() {
    let text = (0..10).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.execute(Command::MoveDown); // verify single works
    assert_eq!(ed.cursor_line, 1);
    ed.cursor_line = 0;
    ed.execute(Command::Repeat(5, Box::new(Command::MoveDown)));
    assert_eq!(ed.cursor_line, 5);
}

#[test]
fn count_3w_moves_3_words() {
    let mut ed = Editor::from_text("one two three four five");
    ed.execute(Command::Repeat(3, Box::new(Command::MoveWordForward)));
    // Should be at "four" (word 3, 0-indexed positions: one=0, two=4, three=8, four=14)
    assert_eq!(ed.cursor_col, 14);
}

#[test]
fn count_5x_deletes_5_chars() {
    let mut ed = Editor::from_text("abcdefgh");
    ed.execute(Command::Repeat(5, Box::new(Command::DeleteCharForward)));
    assert_eq!(ed.buffer.content(), "fgh");
}

#[test]
fn count_3dd_deletes_3_lines() {
    let mut ed = Editor::from_text("line1\nline2\nline3\nline4\nline5");
    ed.execute(Command::Repeat(3, Box::new(Command::DeleteLine)));
    assert_eq!(ed.buffer.content(), "line4\nline5");
}

#[test]
fn count_5yy_yanks_5_lines() {
    let text = "a\nb\nc\nd\ne\nf";
    let mut ed = Editor::from_text(text);
    ed.execute(Command::Repeat(5, Box::new(Command::YankLine)));
    // Repeat(5, YankLine) yanks 5 lines (multi-line yank)
    assert_eq!(ed.register, "a\nb\nc\nd\ne\n");
}

#[test]
fn count_3cc_changes_3_lines() {
    let mut ed = Editor::from_text("line1\nline2\nline3\nline4");
    ed.execute(Command::Repeat(3, Box::new(Command::ChangeLine)));
    // 3 lines deleted, in insert mode
    assert_eq!(ed.mode, EditorMode::Insert);
    assert_eq!(ed.buffer.content(), "\nline4");
}

#[test]
fn count_3_indent() {
    let mut ed = Editor::from_text("a\nb\nc\nd");
    ed.execute(Command::Repeat(3, Box::new(Command::Indent)));
    let content = ed.buffer.content();
    assert!(content.starts_with("    a\n    b\n    c\nd"), "got: {:?}", content);
}

#[test]
fn count_10g_goto_line_10() {
    let text = (1..=20).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.execute(Command::GotoLine(10));
    assert_eq!(ed.cursor_line, 9); // 0-indexed
}

// === Operator motion tests ===

#[test]
fn db_deletes_word_backward() {
    let mut ed = Editor::from_text("hello world");
    ed.cursor_col = 6; // at 'w' of "world"
    ed.execute(Command::DeleteWordBackward);
    // db from 'w' goes back to start of previous word "hello" → deletes "hello "
    assert_eq!(ed.buffer.content(), "world");
    assert_eq!(ed.cursor_col, 0);
}

#[test]
fn d0_deletes_to_line_start() {
    let mut ed = Editor::from_text("hello world");
    ed.cursor_col = 6;
    ed.execute(Command::DeleteToStart);
    assert_eq!(ed.buffer.content(), "world");
    assert_eq!(ed.cursor_col, 0);
}

#[test]
fn big_d_deletes_to_end() {
    let mut ed = Editor::from_text("hello world");
    ed.cursor_col = 5;
    ed.execute(Command::DeleteToEnd);
    assert_eq!(ed.buffer.content(), "hello");
}

#[test]
fn big_c_changes_to_end() {
    let mut ed = Editor::from_text("hello world");
    ed.cursor_col = 5;
    ed.execute(Command::ChangeToEnd);
    assert_eq!(ed.buffer.content(), "hello");
    assert_eq!(ed.mode, EditorMode::Insert);
}

#[test]
fn yw_yanks_word() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::YankWord);
    assert_eq!(ed.register, "hello ");
}

#[test]
fn y_dollar_yanks_to_end() {
    let mut ed = Editor::from_text("hello world");
    ed.cursor_col = 6;
    ed.execute(Command::YankToEnd);
    assert_eq!(ed.register, "world");
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
    assert_eq!(ed.mode, EditorMode::Insert);
    assert_eq!(ed.buffer.content(), " world");
}

#[test]
fn visual_colon_sets_range() {
    let mut ed = Editor::from_text("line1\nline2\nline3\nline4");
    ed.execute(Command::EnterVisualLine);
    ed.execute(Command::MoveDown);
    ed.execute(Command::MoveDown);
    ed.execute(Command::VisualExCommand);
    assert_eq!(ed.mode, EditorMode::Command);
    // command_buf should be pre-filled with range
    assert!(ed.command_buf.starts_with("'<,'>"), "got: {}", ed.command_buf);
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
        ed.status.contains("write") || ed.status.contains("modified") || ed.status.contains("unsaved"),
        "expected dirty warning, got: {:?}", ed.status
    );
}

#[test]
fn ex_q_bang_force_closes() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::InsertChar('x')); // make dirty
    ed.mode = EditorMode::Normal;
    let action = ed.execute(Command::ExCommand("q!".to_string()));
    assert_eq!(action, EditorAction::CloseRequested);
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

// === Paste undo ===

#[test]
fn paste_is_undoable() {
    let mut ed = Editor::from_text("line1\nline2");
    ed.execute(Command::YankLine);
    ed.execute(Command::Paste);
    assert!(ed.buffer.content().contains("line1\nline1"));
    ed.execute(Command::Undo);
    assert_eq!(ed.buffer.content(), "line1\nline2");
}

// === Insert mode features ===

#[test]
fn insert_enter_creates_newline() {
    let mut ed = Editor::from_text("hello");
    ed.mode = EditorMode::Insert;
    ed.cursor_col = 3;
    ed.execute(Command::InsertNewline);
    assert_eq!(ed.buffer.content(), "hel\nlo");
}

#[test]
fn insert_delete_forward() {
    let mut ed = Editor::from_text("hello");
    ed.mode = EditorMode::Insert;
    ed.execute(Command::DeleteCharForward);
    assert_eq!(ed.buffer.content(), "ello");
}

// === 3J join 3 lines ===

#[test]
fn count_3j_joins_3_lines() {
    let mut ed = Editor::from_text("a\nb\nc\nd");
    ed.execute(Command::Repeat(3, Box::new(Command::JoinLines)));
    assert_eq!(ed.buffer.content(), "a b c d");
}

// === Keymap integration: count prefix through keymap ===

use kairn::editor::keymap::Keymap;
use kairn::editor::keymap_vim::VimKeymap;
use txv_core::event::{KeyCode, KeyEvent, KeyMod};

fn key(ch: char) -> KeyEvent {
    KeyEvent { code: KeyCode::Char(ch), modifiers: KeyMod::default() }
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
    ed.cursor_col = 1; // on 'b'
    ed.execute(Command::EnterInsertAfter);
    ed.execute(Command::InsertChar('X'));
    assert_eq!(ed.buffer.content(), "abXc");
}

#[test]
fn big_i_inserts_at_line_start() {
    let mut ed = Editor::from_text("  hello");
    ed.cursor_col = 4;
    ed.execute(Command::EnterInsertLineStart);
    ed.execute(Command::InsertChar('X'));
    // I inserts before first non-blank (col 2)
    assert_eq!(ed.buffer.content(), "  Xhello");
}

#[test]
fn big_a_inserts_at_line_end() {
    let mut ed = Editor::from_text("hello");
    ed.execute(Command::EnterInsertLineEnd);
    ed.execute(Command::InsertChar('!'));
    assert_eq!(ed.buffer.content(), "hello!");
}

#[test]
fn o_opens_line_below() {
    let mut ed = Editor::from_text("line1\nline2");
    ed.execute(Command::EnterInsertBelow);
    ed.execute(Command::InsertChar('X'));
    assert_eq!(ed.mode, EditorMode::Insert);
    assert!(ed.buffer.content().contains("line1\nX\nline2"));
}

#[test]
fn big_o_opens_line_above() {
    let mut ed = Editor::from_text("line1\nline2");
    ed.cursor_line = 1;
    ed.execute(Command::EnterInsertAbove);
    ed.execute(Command::InsertChar('X'));
    assert_eq!(ed.mode, EditorMode::Insert);
    assert!(ed.buffer.content().contains("line1\nX\nline2"));
}

#[test]
fn insert_arrow_keys_move() {
    let mut ed = Editor::from_text("abc\ndef");
    ed.execute(Command::EnterInsertMode);
    ed.execute(Command::MoveRight);
    ed.execute(Command::MoveRight);
    ed.execute(Command::MoveDown);
    assert_eq!(ed.cursor_line, 1);
    assert_eq!(ed.cursor_col, 2);
}

// === Movement: e, ^, Ctrl-D/U, PgUp/PgDn ===

#[test]
fn e_moves_to_word_end() {
    let mut ed = Editor::from_text("hello world");
    ed.execute(Command::MoveWordEnd);
    assert_eq!(ed.cursor_col, 4); // end of "hello"
}

#[test]
fn caret_moves_to_first_nonblank() {
    let mut ed = Editor::from_text("   hello");
    ed.cursor_col = 6;
    ed.execute(Command::MoveFirstNonBlank);
    assert_eq!(ed.cursor_col, 3);
}

#[test]
fn ctrl_d_moves_half_page_down() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.viewport_height = 20;
    ed.execute(Command::HalfPageDown);
    assert_eq!(ed.cursor_line, 10); // half of 20
}

#[test]
fn ctrl_u_moves_half_page_up() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.viewport_height = 20;
    ed.cursor_line = 20;
    ed.execute(Command::HalfPageUp);
    assert_eq!(ed.cursor_line, 10);
}

#[test]
fn page_down_moves_full_page() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.viewport_height = 20;
    ed.execute(Command::PageDown);
    // vim keeps 2 context lines, so moves viewport_height - 2
    assert_eq!(ed.cursor_line, 18);
}

#[test]
fn page_up_moves_full_page() {
    let text = (0..40).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.viewport_height = 20;
    ed.cursor_line = 30;
    ed.execute(Command::PageUp);
    assert_eq!(ed.cursor_line, 12); // 30 - 18
}

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
    assert!(ed.buffer.content().starts_with('\n') || ed.buffer.content() == "world" || ed.buffer.content() == "\nworld");
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
