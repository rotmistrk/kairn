//! Test helpers for kairn integration tests.

use std::path::Path;

use tempfile::TempDir;
use txv_core::geometry::Rect;
use txv_core::run::{run_cycles, MockBackend};
use txv_core::view::View;

use kairn::app::App;

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

/// Create App + MockBackend for a temp project.
#[allow(dead_code)]
pub fn setup(dir: &Path, width: u16, height: u16) -> (App, MockBackend) {
    let backend = MockBackend::new(width, height);
    let mut app = App::new(dir.to_path_buf());
    app.set_bounds(Rect::new(0, 0, width, height));
    (app, backend)
}

/// Run cycles and return screen text.
#[allow(dead_code)]
pub fn run_and_capture(
    app: &mut App,
    backend: &mut MockBackend,
    cycles: usize,
) -> String {
    run_cycles(app, backend, cycles);
    backend.screen_text()
}

/// Find the cursor position on screen by scanning for the reversed cell
/// in the editor area (center slot). Returns (line, col) in editor coordinates
/// where line is the 0-indexed buffer line (read from gutter) and col is
/// the column within the line content.
#[allow(dead_code)]
pub fn cursor_at(be: &MockBackend) -> Option<(usize, usize)> {
    let surface = be.surface()?;
    let w = surface.width();
    let h = surface.height();

    // Find the reversed cell — skip row 0 (chrome) and last row (status)
    // Also skip the left panel area (tree is ~24 cols + 1 divider)
    for y in 1..h.saturating_sub(1) {
        for x in 0..w {
            let cell = surface.cell(x, y);
            if cell.style.attrs.reverse {
                // Check this is in the editor area (has gutter with Ansi(8) on this row)
                let editor_x_start = find_editor_x_start(surface, y);
                if editor_x_start == 0 && x < 25 {
                    // This is likely the tree cursor, skip
                    continue;
                }
                let gutter_w = find_gutter_width(surface, y, editor_x_start);
                let content_x = editor_x_start + gutter_w;

                if x < content_x {
                    // Cursor is in the gutter area — shouldn't happen, skip
                    continue;
                }

                let col = (x - content_x) as usize;
                // Read the line number from the gutter on this row
                let line = read_line_number(surface, y, editor_x_start, gutter_w)
                    .unwrap_or(0)
                    .saturating_sub(1); // convert 1-indexed to 0-indexed
                return Some((line, col));
            }
        }
    }
    None
}

/// Read the line number displayed in the gutter at the given row.
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

/// Find where the editor slot starts on a given row (first cell with Ansi(8) fg = gutter).
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

/// Find gutter width: count cells from editor_x_start that have Ansi(8) fg.
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
