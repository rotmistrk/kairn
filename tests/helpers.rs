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
