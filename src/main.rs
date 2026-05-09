//! kairn — TUI IDE entry point.

use std::path::PathBuf;

use clap::Parser;
use txv_core::geometry::Rect;
use txv_core::run::run;
use txv_core::view::View;
use txv_render::backend::CrosstermBackend;
use txv_render::color::detect_color_mode;

use kairn::app::App;

#[derive(Parser)]
#[command(name = "kairn", about = "TUI IDE")]
struct Cli {
    /// Directory to open
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root_dir = std::fs::canonicalize(&cli.path)?;

    let color_mode = detect_color_mode();
    let mut backend = CrosstermBackend::new(color_mode);

    let (w, h) = txv_core::run::Backend::size(&backend);
    let mut app = App::new(root_dir);
    app.set_bounds(Rect::new(0, 0, w, h));

    run(&mut app, &mut backend);
    Ok(())
}
