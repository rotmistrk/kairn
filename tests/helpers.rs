//! Test harness — uses Program (same code path as real app).

use std::path::Path;

use tempfile::TempDir;
use txv_core::event::{KeyCode, KeyMod};
use txv_core::program::Program;
use txv_core::run::MockBackend;

use kairn::completer::AppCompleter;
use kairn::handler::{build_desktop, handle_command, AppState};
use kairn::status::KairnStatusBar;

/// Test harness that mirrors the real app exactly.
pub struct TestHarness {
    pub program: Program,
    pub backend: MockBackend,
    pub state: AppState,
}

impl TestHarness {
    /// Create a new test harness for the given project directory.
    /// Same setup as main.rs: StatusBar + Desktop + AppState.
    pub fn new(root_dir: &Path) -> Self {
        let desktop = build_desktop(root_dir);
        let mut status = KairnStatusBar::new();
        status.set_completer(Box::new(AppCompleter::new(root_dir.to_path_buf())));
        let program = Program::new(Box::new(status), Box::new(desktop));
        let backend = MockBackend::new(80, 24);
        let state = AppState::new(root_dir.to_path_buf());
        Self { program, backend, state }
    }

    /// Create with custom dimensions.
    pub fn with_size(root_dir: &Path, width: u16, height: u16) -> Self {
        let desktop = build_desktop(root_dir);
        let mut status = KairnStatusBar::new();
        status.set_completer(Box::new(AppCompleter::new(root_dir.to_path_buf())));
        let program = Program::new(Box::new(status), Box::new(desktop));
        let backend = MockBackend::new(width, height);
        let state = AppState::new(root_dir.to_path_buf());
        Self { program, backend, state }
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
        self.program.run_cycles(&mut self.backend, &mut |ctx| {
            handle_command(ctx, state);
        }, n);
    }

    pub fn screen_text(&self) -> String {
        self.backend.screen_text()
    }

    pub fn contains(&self, text: &str) -> bool {
        self.backend.contains(text)
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

/// Find the cursor position by scanning the rendered surface for the
/// cell with reverse attribute in the editor area. Returns (line, col)
/// in buffer coordinates (line number read from gutter).
#[allow(dead_code)]
pub fn cursor_at(h: &TestHarness) -> Option<(usize, usize)> {
    let surface = h.backend.surface()?;
    let w = surface.width();
    let height = surface.height();

    for y in 1..height.saturating_sub(1) {
        for x in 0..w {
            let cell = surface.cell(x, y);
            if cell.style.attrs.reverse {
                let editor_x_start = find_editor_x_start(surface, y);
                if editor_x_start == 0 && x < 25 {
                    continue; // tree cursor
                }
                let gutter_w = find_gutter_width(surface, y, editor_x_start);
                let content_x = editor_x_start + gutter_w;
                if x < content_x {
                    continue;
                }
                let col = (x - content_x) as usize;
                let line = read_line_number(surface, y, editor_x_start, gutter_w)
                    .unwrap_or(0)
                    .saturating_sub(1);
                return Some((line, col));
            }
        }
    }
    None
}

fn find_editor_x_start(surface: &txv_core::surface::Surface, y: u16) -> u16 {
    use txv_core::cell::Color;
    for x in 0..surface.width() {
        let cell = surface.cell(x, y);
        if cell.style.fg == Color::Ansi(8) && cell.ch.is_ascii_digit() {
            return x;
        }
    }
    0
}

fn find_gutter_width(surface: &txv_core::surface::Surface, y: u16, start_x: u16) -> u16 {
    use txv_core::cell::Color;
    let mut w = 0;
    for x in start_x..surface.width() {
        let cell = surface.cell(x, y);
        if cell.style.fg == Color::Ansi(8) {
            w += 1;
        } else {
            break;
        }
    }
    w
}

fn read_line_number(surface: &txv_core::surface::Surface, y: u16, start_x: u16, gutter_w: u16) -> Option<usize> {
    let mut num_str = String::new();
    for x in start_x..start_x + gutter_w {
        let ch = surface.cell(x, y).ch;
        if ch.is_ascii_digit() {
            num_str.push(ch);
        }
    }
    num_str.parse().ok()
}
