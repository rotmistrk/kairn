#![deny(clippy::unwrap_used, clippy::expect_used)]

//! rusticle-tk — TUI application framework with rusticle scripting.
//!
//! Usage:
//! - `rusticle-tk script.tcl` — run a script file
//! - `rusticle-tk -e 'script'` — run inline script
//! - `rusticle-tk -i` — interactive REPL (stub)

use std::process;

use rusticle_tk::{event_mgr, tk_bridge, widget_mgr};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(&args) {
        eprintln!("rusticle-tk: {e}");
        process::exit(1);
    }
}

/// Parse CLI args and execute.
fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("usage: rusticle-tk <script.tcl> | -e 'script' | -i".into());
    }
    match args[0].as_str() {
        "-e" => {
            let script = args.get(1).ok_or("missing script after -e")?;
            run_script(script, args)
        }
        "-i" => Err("interactive REPL not yet implemented".into()),
        path => {
            let content =
                std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
            run_script(&content, args)
        }
    }
}

/// Execute a rusticle script with the TK bridge.
fn run_script(script: &str, args: &[String]) -> Result<(), String> {
    let mut interp = rusticle::interpreter::Interpreter::new();

    // Set argv for the script
    let argv: Vec<rusticle::value::TclValue> = args
        .iter()
        .map(|s| rusticle::value::TclValue::Str(s.clone()))
        .collect();
    interp.set_var("argv", rusticle::value::TclValue::List(argv));

    let shared = tk_bridge::register_all(&mut interp);
    interp.eval(script).map_err(|e| format!("{e}"))?;

    // Print any captured output (from puts in non-TUI mode)
    for line in interp.get_output() {
        print!("{line}");
    }

    // If the script called `app run`, start the event loop
    let needs_run = shared
        .lock()
        .map(|st| st.run_requested && !st.has_run)
        .unwrap_or(false);
    if needs_run {
        // Auto-focus the first focusable widget if none set
        auto_focus(&shared);
        event_mgr::run_event_loop(&mut interp, &shared)?;
    }
    Ok(())
}

/// Set focus to the first widget if none is focused.
fn auto_focus(shared: &tk_bridge::Shared) {
    let Ok(mut st) = shared.lock() else { return };
    if st.focused.is_some() {
        return;
    }
    // Pick the first widget that is focusable (not statusbar/tabbar/progress)
    let first = st
        .widgets
        .ids()
        .find(|id| {
            matches!(
                st.widgets.get(id),
                Some(
                    widget_mgr::WidgetEntry::Text(_)
                        | widget_mgr::WidgetEntry::List(_)
                        | widget_mgr::WidgetEntry::Tree(_)
                        | widget_mgr::WidgetEntry::Input(_)
                        | widget_mgr::WidgetEntry::Table(_)
                )
            )
        })
        .cloned();
    st.focused = first;
}
