// === TextArea search and file tree hidden files tests ===

mod helpers;

use helpers::TestHarness;
use txv_core::event::{KeyCode, KeyMod};
use txv_core::prelude::*;
use txv_widgets::TextArea;

// ─── TextArea search ───────────────────────────────────────────────

#[test]
fn text_area_search_finds_matches() {
    let mut ta = TextArea::new();
    ta.set_content("line one\nline two\nline three\nfour");
    ta.search("two");
    assert_eq!(ta.search_matches, vec![1]);
    assert_eq!(ta.current_match, 0);
}

#[test]
fn text_area_search_multiple_matches() {
    let mut ta = TextArea::new();
    ta.set_content("aaa\nbbb\naaa\nccc\naaa");
    ta.search("aaa");
    assert_eq!(ta.search_matches, vec![0, 2, 4]);
}

#[test]
fn text_area_next_prev_match() {
    let mut ta = TextArea::new();
    ta.set_content("x\ny\nx\ny\nx");
    ta.search("x");
    assert_eq!(ta.current_match, 0);
    ta.next_match();
    assert_eq!(ta.current_match, 1);
    ta.next_match();
    assert_eq!(ta.current_match, 2);
    ta.next_match();
    // Wraps around
    assert_eq!(ta.current_match, 0);
    ta.prev_match();
    assert_eq!(ta.current_match, 2);
}

#[test]
fn text_area_search_empty_query_clears() {
    let mut ta = TextArea::new();
    ta.set_content("hello\nworld");
    ta.search("hello");
    assert_eq!(ta.search_matches.len(), 1);
    ta.search("");
    assert!(ta.search_matches.is_empty());
}

#[test]
fn text_area_slash_key_activates_search() {
    let mut ta = TextArea::new();
    ta.set_content("alpha\nbeta\ngamma");
    ta.set_bounds(Rect {
        x: 0,
        y: 0,
        w: 40,
        h: 10,
    });
    let mut queue = EventQueue::new();
    // Press /
    let slash = Event::Key(KeyEvent {
        code: KeyCode::Char('/'),
        modifiers: KeyMod::default(),
    });
    let result = ta.handle(&slash, &mut queue);
    assert_eq!(result, HandleResult::Consumed);
    // Type "beta"
    for ch in "beta".chars() {
        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyMod::default(),
        });
        ta.handle(&ev, &mut queue);
    }
    // Press Enter to confirm
    let enter = Event::Key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyMod::default(),
    });
    ta.handle(&enter, &mut queue);
    assert_eq!(ta.search_matches, vec![1]);
    assert_eq!(ta.search_query, "beta");
}

#[test]
fn text_area_search_esc_cancels() {
    let mut ta = TextArea::new();
    ta.set_content("foo\nbar");
    ta.set_bounds(Rect {
        x: 0,
        y: 0,
        w: 40,
        h: 10,
    });
    let mut queue = EventQueue::new();
    // Activate search
    let slash = Event::Key(KeyEvent {
        code: KeyCode::Char('/'),
        modifiers: KeyMod::default(),
    });
    ta.handle(&slash, &mut queue);
    // Type something
    let ev = Event::Key(KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyMod::default(),
    });
    ta.handle(&ev, &mut queue);
    // Esc cancels
    let esc = Event::Key(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyMod::default(),
    });
    ta.handle(&esc, &mut queue);
    // No search performed
    assert!(ta.search_matches.is_empty());
}

// ─── Hidden files toggle ───────────────────────────────────────────

#[test]
fn hidden_files_always_visible() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("visible.txt"), "v").unwrap();
    std::fs::write(dir.path().join(".hidden"), "h").unwrap();

    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let screen = h.screen_text();
    assert!(screen.contains("visible.txt"), "visible file missing: {screen}");
    assert!(screen.contains(".hidden"), "hidden file should always show: {screen}");
}
