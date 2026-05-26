//! Event manager — runs the TUI via txv_core::program::Program.
//!
//! Key bindings from the script become KeyLabelItem entries in the StatusBar.
//! The command handler evaluates rusticle callbacks.

use std::collections::HashMap;

use txv_core::prelude::*;
use txv_core::program::{CommandContext, Program};
use txv_core::run::Backend;
use txv_core::status_bar::{Gravity, StatusBar, StatusSlot};
use txv_widgets::{ClockView, KeyLabelView};

use crate::desktop::TkDesktop;
use crate::keyspec::{format_key_label, parse_keyspec};

use rusticle::interpreter::Interpreter;

/// Command IDs for script-defined bindings (start above txv built-ins).
pub const CM_SCRIPT_BASE: CommandId = 200;

/// A timer definition from the `after` command.
pub struct TimerDef {
    /// Delay in milliseconds.
    pub delay_ms: u64,
    /// Whether this timer repeats.
    pub repeat: bool,
    /// Script to evaluate when the timer fires.
    pub script: String,
}

/// Event callback state collected during script evaluation.
pub struct EventState {
    /// Global key bindings: keyspec → script.
    pub key_bindings: HashMap<String, String>,
    /// Widget event handlers: (widget_id, event_name) → proc name.
    pub widget_handlers: HashMap<(String, String), String>,
    /// Timer definitions.
    pub timers: Vec<TimerDef>,
    /// Quit handler script.
    pub quit_handler: Option<String>,
    /// Whether quit was requested.
    pub quit_requested: bool,
}

impl EventState {
    /// Create empty event state.
    pub fn new() -> Self {
        Self {
            key_bindings: HashMap::new(),
            widget_handlers: HashMap::new(),
            timers: Vec::new(),
            quit_handler: None,
            quit_requested: false,
        }
    }
}

impl Default for EventState {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a StatusBar from the script's key bindings.
fn build_status_bar(events: &EventState) -> StatusBar {
    let mut bar = StatusBar::new();
    let mut bindings: Vec<(&String, &String)> = events.key_bindings.iter().collect();
    bindings.sort_by_key(|(k, _)| k.as_str());
    for (i, (keyspec, _)) in bindings.iter().enumerate() {
        if let Some(key_event) = parse_keyspec(keyspec) {
            let cmd_id = CM_SCRIPT_BASE + i as u16;
            let label = format_key_label(keyspec);
            bar.add(StatusSlot::new(Box::new(KeyLabelView::new(key_event, cmd_id, label))));
        }
    }
    bar.add(
        StatusSlot::new(Box::new(ClockView::new(60)))
            .priority(2)
            .gravity(Gravity::Right),
    );
    bar
}

/// Run the TUI event loop using Program.
pub fn run_event_loop(
    interp: &mut Interpreter,
    events: &mut EventState,
    desktop: TkDesktop,
    backend: &mut dyn Backend,
) -> Result<(), String> {
    let status_bar = build_status_bar(events);
    let mut program = Program::new(Box::new(status_bar), Box::new(desktop));

    let mut bindings: Vec<(&String, &String)> = events.key_bindings.iter().collect();
    bindings.sort_by_key(|(k, _)| k.as_str());
    let binding_scripts: Vec<String> = bindings.iter().map(|(_, s)| s.to_string()).collect();

    let quit_handler = events.quit_handler.clone();
    let interp_ptr = interp as *mut Interpreter;

    program.run(backend, |ctx| {
        let ip = unsafe { &mut *interp_ptr };
        handle_command(ip, &binding_scripts, &quit_handler, ctx);
    });

    Ok(())
}

/// Handle commands emitted by the StatusBar.
fn handle_command(
    interp: &mut Interpreter,
    binding_scripts: &[String],
    quit_handler: &Option<String>,
    ctx: &mut CommandContext,
) {
    let cmd = ctx.command;
    if cmd >= CM_SCRIPT_BASE {
        let idx = (cmd - CM_SCRIPT_BASE) as usize;
        if let Some(script) = binding_scripts.get(idx) {
            let _ = interp.eval(script);
        }
    }
    let _ = quit_handler;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_state_default() {
        let es = EventState::new();
        assert!(es.key_bindings.is_empty());
        assert!(es.widget_handlers.is_empty());
        assert!(es.timers.is_empty());
        assert!(!es.quit_requested);
    }
}
