// Deny unwrap/expect in non-test code — forces proper error handling.
#![deny(clippy::unwrap_used, clippy::expect_used)]
// Allow dead code during scaffold phase — remove once features are wired up.
#![allow(dead_code)]
// Allow unused imports in legacy modules we don't modify.
#![allow(unused_imports)]

mod app;
mod buffer;
mod capture;
mod cli;
mod config;
mod content_search;
mod csv_table;
mod diff;
mod editor;
mod git;
mod help;
mod highlight;
mod keymap;
mod layout;
mod nav;
mod overlay;
mod panel;
mod rusticle_bridge;
mod search;
mod session;
mod styled;
mod tab;
mod termbuf;
mod tree;

use std::io;
use std::panic;

use anyhow::Result;
use crossterm::terminal;
use txv::screen::Screen;
use txv_widgets::EventLoop;

use app::App;
use cli::Cli;

fn main() {
    // Install panic handler that restores terminal before printing.
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), terminal::LeaveAlternateScreen);
        default_hook(info);
    }));

    if let Err(e) = run() {
        eprintln!("kairn: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse_args();
    let workspace = cli.resolve_path();

    // Prevent nested instances.
    if std::env::var("KAIRN_PID").is_ok() {
        anyhow::bail!(
            "kairn is already running (KAIRN_PID set). \
             Nested instances are not supported."
        );
    }
    std::env::set_var("KAIRN_PID", std::process::id().to_string());

    let (cols, rows) = terminal::size()?;
    let screen = Screen::new(cols, rows);
    let mut event_loop = EventLoop::new(screen);
    event_loop.set_tick_ms(50);

    let mut app = App::new(workspace, cli.config.as_deref());

    event_loop.run(|ctx| {
        // Drain any pending pollers from the app and add them.
        // Note: we can't add pollers to EventLoop from inside run().
        // Instead, App handles PTY I/O via background threads + channels.
        app.tick(ctx)
    })?;

    Ok(())
}
