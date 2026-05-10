// === Viewport scrolling ===

mod helpers;

use helpers::{cursor_at, temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn scrolls_down_when_cursor_moves_past_viewport() {
    // Create file with more lines than viewport height
    let content: String = (1..=50).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let dir = temp_project(&[("t.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 10);
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Move down past viewport (viewport is ~8 lines for editor area)
    for _ in 0..20 {
        h.inject_key(KeyCode::Char('j'), KeyMod::default());
    }
    h.run_cycles(1);
    // line21 should be visible (cursor is there)
    assert!(h.contains("line21"), "expected line21 visible after scrolling");
    // Cursor should be on line 21 (0-indexed: 20)
    assert_eq!(cursor_at(&h), Some((20, 0)));
}

#[test]
fn scrolls_up_when_cursor_moves_above_viewport() {
    let content: String = (1..=50).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let dir = temp_project(&[("t.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 10);
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Go to line 30
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("30\n");
    h.run_cycles(1);
    // Now move up past viewport
    for _ in 0..10 {
        h.inject_key(KeyCode::Char('k'), KeyMod::default());
    }
    h.run_cycles(1);
    // Cursor should be at line 20, which should be visible
    assert_eq!(cursor_at(&h), Some((19, 0)));
    assert!(h.contains("line20"));
}

#[test]
fn gg_scrolls_to_top() {
    let content: String = (1..=50).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let dir = temp_project(&[("t.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 10);
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Go to bottom
    h.inject_key(KeyCode::Char('G'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("line50"));
    // gg goes to top
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("line1"));
}
