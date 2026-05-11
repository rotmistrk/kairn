mod helpers;

use helpers::{cursor_at, temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn colon_w_saves_file() {
    let dir = temp_project(&[("t.txt", "original")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_str("NEW ");
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("w\n");
    h.run_cycles(1);
    assert!(h.contains("NEW original"));
}

#[test]
fn colon_q_closes_buffer() {
    use kairn::editor::ex::{parse_ex_full, ExCommand};
    assert_eq!(parse_ex_full("q", 0, 10), Some(ExCommand::Quit));
    assert_eq!(parse_ex_full("w", 0, 10), Some(ExCommand::Save));
}

#[test]
fn colon_wq_saves_and_closes() {
    use kairn::editor::ex::{parse_ex_full, ExCommand};
    assert_eq!(parse_ex_full("wq", 0, 10), Some(ExCommand::SaveQuit));
}

#[test]
fn slash_searches_forward() {
    let dir = temp_project(&[("t.txt", "hello world\nfoo bar")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    assert!(h.contains("hello world"));
    assert!(h.contains("foo bar"));
}

#[test]
fn editor_shows_line_numbers() {
    let dir = temp_project(&[("t.txt", "aaa\nbbb\nccc")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    assert!(h.contains("1 aaa"));
    assert!(h.contains("2 bbb"));
    assert!(h.contains("3 ccc"));
}

#[test]
fn ctrl_r_redoes() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('x'), KeyMod::default());
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.inject_key(
        KeyCode::Char('r'),
        KeyMod {
            ctrl: true,
            alt: false,
            shift: false,
        },
    );
    h.run_cycles(1);
    assert!(h.contains("ello"));
    assert!(!h.contains("hello"));
}

// --- BUG 2: Editor : mode accepts typing and executes ---
#[test]
fn colon_mode_accepts_typing_and_executes() {
    let content: String = (1..=50).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let dir = temp_project(&[("big.txt", &content)]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("42\n");
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((41, 0)));
}

// --- BUG 1: Editor : mode prompt must be visible ---
#[test]
fn colon_mode_shows_prompt() {
    let dir = temp_project(&[("t.txt", "hello\nworld")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Press : to enter command mode
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.run_cycles(1);
    // The screen should show a ":" prompt somewhere in the editor area
    let screen = h.screen_text();
    // Check rows in the editor area (not the status bar at row 23)
    let has_prompt = (1..23u16).any(|y| {
        let row = h.row(y);
        row.contains(':')
    });
    assert!(has_prompt, "expected : prompt visible in editor area, got:\n{}", screen);
}
