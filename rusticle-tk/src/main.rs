#![deny(clippy::unwrap_used, clippy::expect_used)]

//! rusticle-tk — TUI application framework with rusticle scripting.

use std::process;

use txv_render::backend::CrosstermBackend;
use txv_render::color::detect_color_mode;

use rusticle_tk::tk_bridge;

fn main() {
    // Panic handler: restore terminal before crashing
    std::panic::set_hook(Box::new(|info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        eprintln!("\n\x1b[1;31mrusticle-tk panicked!\x1b[0m");
        eprintln!("{info}");
    }));

    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("rusticle-tk: {e}");
        process::exit(1);
    }
}

/// Parse CLI args and execute.
fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: rusticle-tk <script.tcl> | -e 'script'".into());
    }
    match args[0].as_str() {
        "-e" => {
            let script = args.get(1).ok_or("missing script after -e")?;
            run_script(script)
        }
        path => {
            let content = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
            run_script(&content)
        }
    }
}

/// Execute a rusticle script with the TK bridge.
fn run_script(script: &str) -> Result<(), String> {
    let mut interp = rusticle::interpreter::Interpreter::new();
    let shared = tk_bridge::register_all(&mut interp);

    interp.eval(script).map_err(|e| format!("{e}"))?;

    // Print any captured output (from puts in non-TUI mode)
    for line in interp.get_output() {
        print!("{line}");
    }

    // If the script called `app run`, start the event loop
    let needs_run = shared.lock().map(|st| st.run_requested && !st.has_run).unwrap_or(false);

    if needs_run {
        let (mut events, desktop) = {
            let mut st = shared.lock().map_err(|_| "lock poisoned")?;
            st.has_run = true;
            let desktop = std::mem::take(&mut st.desktop);
            let events = std::mem::take(&mut st.events);
            (events, desktop)
        };
        let color_mode = detect_color_mode();
        let mut backend = CrosstermBackend::new(color_mode);
        rusticle_tk::event_mgr::run_event_loop(&mut interp, &mut events, desktop, &mut backend)?;
    }
    Ok(())
}
