# Step 06: Behavioral Scenario-Driven Test Suite

**Reference**: `doc/f4-design/v-013-txv-architecture.md` (MockBackend)
**Depends on**: Steps 01-05 complete

## What this is

Automated behavioral tests that verify kairn works correctly without
a real terminal. Each test creates a temp project, injects key sequences,
and asserts what's rendered on screen and what state changed.

## Boundary

- **Creates**: `tests/` directory in kairn, helpers in txv-core
- **May modify**: txv-core (add run_cycles, surface assertion helpers)
- **Does NOT touch**: txv-render/, txv-widgets/ internals

## Infrastructure (in txv-core)

### Add to txv-core/src/run.rs:

```rust
/// Run N event-loop cycles with a MockBackend (for testing).
pub fn run_cycles(root: &mut dyn View, backend: &mut MockBackend, n: usize) {
    let mut queue = EventQueue::new();
    let (w, h) = backend.size();
    let mut surface = Surface::new(w, h);

    for _ in 0..n {
        // Process all injected events
        while let Some(event) = backend.poll_event(Duration::ZERO) {
            if let Event::Resize(nw, nh) = &event {
                surface = Surface::new(*nw, *nh);
            }
            root.handle(&event, &mut queue);
        }
        // Dispatch queued commands
        let events = queue.drain();
        for ev in events {
            root.handle(&ev, &mut queue);
        }
        // Draw
        surface.fill(' ', Style::default());
        root.draw(&mut surface);
        backend.flush(&surface);
    }
}
```

### Add to MockBackend:

```rust
impl MockBackend {
    /// Inject a sequence of characters as key events.
    pub fn inject_str(&mut self, s: &str) {
        for ch in s.chars() {
            match ch {
                '\n' => self.inject_key(KeyCode::Enter, KeyMod::default()),
                '\x1b' => self.inject_key(KeyCode::Esc, KeyMod::default()),
                '\t' => self.inject_key(KeyCode::Tab, KeyMod::default()),
                c => self.inject_key(KeyCode::Char(c), KeyMod::default()),
            }
        }
    }

    /// Get the full screen as a string (rows joined by newline).
    pub fn screen_text(&self) -> String;

    /// Check if any row contains the given text.
    pub fn contains(&self, text: &str) -> bool;

    /// Get text of a specific row.
    pub fn row(&self, y: u16) -> String;

    /// Get text of the top row (tab bar).
    pub fn tab_bar(&self) -> String { self.row(0) }

    /// Get text of the bottom row (status bar).
    pub fn status_bar(&self) -> String { self.row(self.height - 1) }
}
```

## Test helper (in tests/helpers/mod.rs)

```rust
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Create a temp project with given files.
pub fn temp_project(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for (path, content) in files {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, content).unwrap();
    }
    dir
}

/// Create App + MockBackend for a temp project.
pub fn setup(dir: &Path, width: u16, height: u16) -> (App, MockBackend) {
    let backend = MockBackend::new(width, height);
    let app = App::new(dir);
    (app, backend)
}

/// Run cycles and return screen text.
pub fn run_and_capture(app: &mut App, backend: &mut MockBackend, cycles: usize) -> String {
    run_cycles(app, backend, cycles);
    backend.screen_text()
}
```

## Test scenarios

### tests/scenarios/tree_navigation.rs

```rust
#[test] fn tree_shows_files_on_start()
#[test] fn tree_j_moves_cursor_down()
#[test] fn tree_k_moves_cursor_up()
#[test] fn tree_enter_on_dir_expands()
#[test] fn tree_enter_on_expanded_dir_collapses()
#[test] fn tree_enter_on_file_emits_open()
#[test] fn tree_respects_gitignore()
#[test] fn tree_dirs_sort_before_files()
#[test] fn tree_shows_nested_structure()
```

### tests/scenarios/file_open.rs

```rust
#[test] fn enter_opens_file_in_center_slot()
#[test] fn open_same_file_twice_focuses_existing_tab()
#[test] fn tab_bar_shows_filename()
#[test] fn multiple_files_create_multiple_tabs()
#[test] fn center_slot_shows_file_content()
#[test] fn file_content_has_line_numbers()
```

### tests/scenarios/editor_vim_movement.rs

```rust
#[test] fn h_moves_left()
#[test] fn l_moves_right()
#[test] fn j_moves_down()
#[test] fn k_moves_up()
#[test] fn w_moves_word_forward()
#[test] fn b_moves_word_backward()
#[test] fn zero_moves_to_line_start()
#[test] fn dollar_moves_to_line_end()
#[test] fn gg_moves_to_file_start()
#[test] fn G_moves_to_file_end()
#[test] fn ctrl_d_half_page_down()
#[test] fn ctrl_u_half_page_up()
```

### tests/scenarios/editor_vim_editing.rs

```rust
#[test] fn i_enters_insert_mode()
#[test] fn typing_inserts_text()
#[test] fn esc_returns_to_normal()
#[test] fn x_deletes_char()
#[test] fn dd_deletes_line()
#[test] fn dw_deletes_word()
#[test] fn o_opens_line_below()
#[test] fn O_opens_line_above()
#[test] fn u_undoes_last_edit()
#[test] fn ctrl_r_redoes()
#[test] fn p_pastes_after()
#[test] fn yy_yanks_line()
```

### tests/scenarios/editor_vim_ex.rs

```rust
#[test] fn colon_w_saves_file()
#[test] fn colon_q_closes_buffer()
#[test] fn colon_wq_saves_and_closes()
#[test] fn colon_number_goes_to_line()
#[test] fn colon_s_substitutes()
#[test] fn slash_searches_forward()
#[test] fn n_goes_to_next_match()
#[test] fn N_goes_to_prev_match()
```

### tests/scenarios/slot_navigation.rs

```rust
#[test] fn f2_focuses_tree_slot()
#[test] fn f3_focuses_center_slot()
#[test] fn f4_focuses_right_slot()
#[test] fn f5_toggles_zoom()
#[test] fn zoom_shows_only_focused_slot()
#[test] fn unzoom_restores_layout()
#[test] fn ctrl_shift_left_prev_tab()
#[test] fn ctrl_shift_right_next_tab()
```

### tests/scenarios/tab_management.rs

```rust
#[test] fn new_file_creates_tab()
#[test] fn close_tab_removes_it()
#[test] fn tab_bar_shows_all_tabs()
#[test] fn active_tab_highlighted()
#[test] fn switch_tab_changes_content()
```

### tests/scenarios/status_bar.rs

```rust
#[test] fn status_bar_shows_key_hints()
#[test] fn status_bar_shows_mode_indicator()
#[test] fn status_bar_shows_filename()
#[test] fn status_bar_shows_line_col()
#[test] fn modified_indicator_shown_after_edit()
```

### tests/scenarios/command_mode.rs

```rust
#[test] fn alt_x_opens_prompt()
#[test] fn typing_in_prompt_shows_text()
#[test] fn esc_cancels_prompt()
#[test] fn enter_executes_command()
#[test] fn help_command_opens_help()
#[test] fn quit_command_exits()
#[test] fn tab_completes_command()
```

### tests/scenarios/chrome.rs

```rust
#[test] fn top_line_has_box_drawing()
#[test] fn vertical_dividers_between_slots()
#[test] fn tab_names_in_top_line()
#[test] fn active_tab_different_style()
#[test] fn no_outer_borders()
#[test] fn bottom_divider_when_bottom_visible()
```

### tests/scenarios/resize.rs

```rust
#[test] fn resize_recomputes_layout()
#[test] fn small_terminal_hides_right_slot()
#[test] fn tree_resize_with_ctrl_arrows()
```

### tests/scenarios/select_unselect.rs

```rust
#[test] fn focused_slot_has_visual_indicator()
#[test] fn switching_focus_updates_indicator()
#[test] fn tree_cursor_visible_when_focused()
#[test] fn tree_cursor_hidden_when_unfocused()
```

## How to run

```bash
cargo test -p kairn --test '*'
# Or specific scenario:
cargo test -p kairn --test scenarios::file_open
```

## Verification

```bash
cargo test --workspace    # all pass
dupfinder tests/          # no duplicate test code >5 lines
```

## Do NOT

- Do NOT use real terminals (MockBackend only)
- Do NOT share state between tests (each creates own tempdir)
- Do NOT test visual aesthetics (colors, timing) — only correctness
- Do NOT skip edge cases (empty files, empty dirs, long filenames)
- Do NOT write tests that depend on execution order
