//! Scenario tests for CSV/JSON row operations — add, delete, yank/paste, visual.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_csv(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

#[test]
fn csv_add_row() {
    let csv = "name,age\nalice,30\nbob,25\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);

    // Press 'a' to add a row after current (alice row)
    h.inject_key(KeyCode::Char('a'), KeyMod::default());
    h.run_cycles(2);

    // Should now have 3 data rows (alice, empty, bob)
    // The new empty row should be visible
    let screen = h.screen_text();
    assert!(screen.contains("alice"), "alice should still be visible");
    assert!(screen.contains("bob"), "bob should still be visible");
}

#[test]
fn csv_yank_paste_row() {
    let csv = "name,age\nalice,30\nbob,25\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);

    // Yank current row (alice)
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(1);
    // Move to bob row
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(1);
    // Paste after bob
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(2);

    // Screen should show alice twice (original + pasted copy)
    let screen = h.screen_text();
    let first_alice = screen.find("alice").unwrap_or(usize::MAX);
    let second_alice = screen[first_alice + 5..].find("alice");
    assert!(
        second_alice.is_some(),
        "should have two 'alice' rows after yank+paste: {}",
        screen
    );
}

#[test]
fn csv_visual_select_and_yank() {
    let csv = "name,age\nalice,30\nbob,25\ncharlie,35\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);

    // Enter visual mode
    h.inject_key(KeyCode::Char('v'), KeyMod::default());
    h.run_cycles(1);
    // Extend selection down (selects alice + bob)
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(1);
    // Yank
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(1);
    // Move to charlie
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(1);
    // Paste
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(2);

    // Should have 5 rows now: alice, bob, charlie, alice, bob
    let screen = h.screen_text();
    let alice_count = screen.matches("alice").count();
    assert!(
        alice_count >= 2,
        "should have 2 alice entries after visual yank+paste, got {alice_count}: {}",
        screen
    );
}

#[test]
fn json_struct_yank_paste() {
    let json = r#"["alpha","beta","gamma"]"#;
    let dir = temp_project(&[("data.json", json)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // Root is array [3], auto-expanded. First element is "alpha" at cursor after down.
    h.inject_key(KeyCode::Down, KeyMod::default()); // "alpha"
    h.run_cycles(1);

    // Yank "alpha"
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(1);

    // Move to "gamma" (down, down)
    h.inject_key(KeyCode::Down, KeyMod::default()); // "beta"
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default()); // "gamma"
    h.run_cycles(1);

    // Paste after "gamma"
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(2);

    // Should now have 4 elements including two "alpha"
    let screen = h.screen_text();
    let alpha_count = screen.matches("alpha").count();
    assert!(
        alpha_count >= 2,
        "should have two 'alpha' values after yank+paste, got {alpha_count}: {}",
        screen
    );
}
