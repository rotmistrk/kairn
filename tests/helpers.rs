//! Test harness — uses Program (same code path as real app).
//! Test harness — uses the SAME initialization path as main().

#![allow(dead_code)]

use std::path::Path;

use tempfile::TempDir;
use txv_core::event::{KeyCode, KeyMod};
use txv_core::program::Program;
use txv_core::run::MockBackend;

use kairn::handler::{handle_command, AppState};

/// Test harness that uses the exact same app setup as main().
pub struct TestHarness {
    pub program: Program,
    pub backend: MockBackend,
    pub state: AppState,
}

/// Initialize logger for tests (safe to call multiple times).
fn init_test_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
    txv_core::glyphs::set_glyphs(txv_core::glyphs::GlyphSet::nerd());
    std::env::set_var("KAIRN_TEST", "1");
}

impl TestHarness {
    /// Create a new test harness — same init as main().
    pub fn new(root_dir: &Path) -> Self {
        Self::with_size(root_dir, 80, 24)
    }

    /// Create with custom dimensions — same init as main().
    pub fn with_size(root_dir: &Path, width: u16, height: u16) -> Self {
        init_test_logger();
        let (program, state) = kairn::app_init::build_app(root_dir);
        let backend = MockBackend::new(width, height);
        Self {
            program,
            backend,
            state,
        }
    }

    pub fn inject_key(&mut self, code: KeyCode, mods: KeyMod) {
        self.backend.inject_key(code, mods);
    }

    pub fn inject_str(&mut self, s: &str) {
        self.backend.inject_str(s);
    }

    /// Run N cycles of the event loop (same dispatch as real app).
    pub fn run_cycles(&mut self, n: usize) {
        let state = &mut self.state;
        self.program.run_cycles(
            &mut self.backend,
            &mut |ctx| {
                handle_command(ctx, state);
            },
            n,
        );
    }

    /// Directly dispatch a command through the handler (bypasses event loop).
    /// Follow-up commands are left in the sink for run_cycles to process
    /// through the full group dispatch (status bar + desktop).
    pub fn dispatch_command(&mut self, id: u16, data: Option<Box<dyn std::any::Any + Send>>) {
        use txv_core::program::CommandContext;
        let sink = self.program.sink().clone();
        let desktop = self.program.desktop_mut();
        let mut ctx = CommandContext::new(id, &data, &sink, desktop);
        handle_command(&mut ctx, &mut self.state);
    }

    pub fn screen_text(&self) -> String {
        self.backend.screen_text()
    }

    pub fn contains(&self, text: &str) -> bool {
        self.backend.contains(text)
    }

    /// Check content area only (excludes status bar) — use for buffer content assertions.
    pub fn content_contains(&self, text: &str) -> bool {
        self.backend.content_contains(text)
    }

    pub fn row(&self, y: u16) -> String {
        self.backend.row(y)
    }
}

/// Create a temp project with given files.
#[allow(dead_code)]
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

/// Find the cursor position by scanning the rendered buffer for the
/// cell with reverse attribute in the editor area. Returns (line, col)
/// in buffer coordinates (line number read from gutter).
#[allow(dead_code)]
pub fn cursor_at(h: &TestHarness) -> Option<(usize, usize)> {
    let buf = h.backend.buffer()?;
    let w = buf.width();
    let height = buf.height();

    for y in 1..height.saturating_sub(1) {
        for x in 0..w {
            let cell = buf.cell(x, y);
            if cell.style().bg() == txv_core::cell::Color::Ansi(7)
                && cell.style().fg() == txv_core::cell::Color::Ansi(0)
            {
                let editor_x_start = find_editor_x_start(buf, y);
                if editor_x_start == 0 && x < 25 {
                    continue; // tree cursor
                }
                let gutter_w = find_gutter_width(buf, y, editor_x_start);
                let content_x = editor_x_start + gutter_w;
                if x < content_x {
                    continue;
                }
                let col = (x - content_x) as usize;
                let line = read_line_number(buf, y, editor_x_start, gutter_w)
                    .unwrap_or(0)
                    .saturating_sub(1);
                return Some((line, col));
            }
        }
    }
    None
}

fn find_editor_x_start(buf: &txv_core::buffer::Buffer, y: u16) -> u16 {
    use txv_core::cell::Color;
    for x in 0..buf.width() {
        let cell = buf.cell(x, y);
        if cell.style().fg() == Color::Ansi(8) && cell.ch().is_ascii_digit() {
            return x;
        }
    }
    0
}

fn find_gutter_width(buf: &txv_core::buffer::Buffer, y: u16, start_x: u16) -> u16 {
    use txv_core::cell::Color;
    let mut w = 0;
    for x in start_x..buf.width() {
        let cell = buf.cell(x, y);
        if cell.style().fg() == Color::Ansi(8) {
            w += 1;
        } else {
            break;
        }
    }
    w
}

fn read_line_number(buf: &txv_core::buffer::Buffer, y: u16, start_x: u16, gutter_w: u16) -> Option<usize> {
    let mut num_str = String::new();
    for x in start_x..start_x + gutter_w {
        let ch = buf.cell(x, y).ch();
        if ch.is_ascii_digit() {
            num_str.push(ch);
        }
    }
    num_str.parse().ok()
}
