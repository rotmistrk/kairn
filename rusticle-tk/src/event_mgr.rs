//! Event manager — connects EventLoop to rusticle callbacks.
//!
//! The event loop cycle:
//! 1. EventLoop delivers crossterm events via RunContext
//! 2. Key events: check global bindings first, then dispatch to focused widget
//! 3. Widget produces WidgetAction → look up handler → call rusticle proc
//! 4. Timer events → eval rusticle script
//! 5. Render all widgets into their layout rects
//! 6. Flush screen

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use txv::layout::Rect;
use txv::screen::Screen;
use txv_widgets::event_loop::{EventLoop, LoopControl, RunContext};
use txv_widgets::widget::{EventResult, WidgetAction};

use crate::tk_bridge::SharedState;

use rusticle::interpreter::Interpreter;

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
    /// Resize handler script.
    pub resize_handler: Option<String>,
    /// Whether quit was requested.
    pub quit_requested: bool,
    /// Pending notification: (message, duration_ms).
    pub pending_notification: Option<(String, u64)>,
}

impl EventState {
    /// Create empty event state.
    pub fn new() -> Self {
        Self {
            key_bindings: HashMap::new(),
            widget_handlers: HashMap::new(),
            timers: Vec::new(),
            quit_handler: None,
            resize_handler: None,
            quit_requested: false,
            pending_notification: None,
        }
    }
}

impl Default for EventState {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the TUI event loop, rendering widgets and dispatching events.
pub fn run_event_loop(
    interp: &mut Interpreter,
    shared: &Arc<Mutex<SharedState>>,
) -> Result<(), String> {
    let (w, h) = crossterm::terminal::size().map_err(|e| e.to_string())?;
    let screen = Screen::new(w, h);
    let mut event_loop = EventLoop::new(screen);

    // Install timers from script
    install_timers(shared, &mut event_loop);

    // Mark as running
    if let Ok(mut st) = shared.lock() {
        st.has_run = true;
    }

    let shared_clone = Arc::clone(shared);
    let interp_ptr = interp as *mut Interpreter;
    event_loop
        .run(move |ctx: &mut RunContext| {
            // Safety: interp lives for the duration of run_event_loop,
            // and run() blocks until the loop exits.
            let ip = unsafe { &mut *interp_ptr };
            handle_tick(ip, &shared_clone, ctx)
        })
        .map_err(|e| e.to_string())
}

/// Install timer definitions into the event loop.
fn install_timers(shared: &Arc<Mutex<SharedState>>, event_loop: &mut EventLoop) {
    let timer_defs: Vec<TimerDef> = {
        let Ok(mut st) = shared.lock() else { return };
        std::mem::take(&mut st.events.timers)
    };
    for def in timer_defs {
        let script = def.script.clone();
        let sh = Arc::clone(shared);
        event_loop.add_timer(
            def.delay_ms,
            def.repeat,
            Box::new(move || {
                // We can't call interp here (not accessible), so store
                // the script for evaluation in the main loop tick
                if let Ok(mut st) = sh.lock() {
                    st.events.timers.push(TimerDef {
                        delay_ms: 0,
                        repeat: false,
                        script: script.clone(),
                    });
                }
                true
            }),
        );
    }
}

/// Handle one tick of the event loop.
fn handle_tick(
    interp: &mut Interpreter,
    shared: &Arc<Mutex<SharedState>>,
    ctx: &mut RunContext,
) -> LoopControl {
    // Process pending timer scripts
    eval_pending_timers(interp, shared);

    // Handle resize
    for event in &ctx.events {
        if let Event::Resize(w, h) = event {
            ctx.screen.resize(*w, *h);
            eval_handler(interp, shared, "resize_handler");
        }
    }

    // Handle key events
    for event in &ctx.events {
        if let Event::Key(key) = event {
            if !handle_key(interp, shared, *key) {
                // Key not consumed by bindings, dispatch to focused widget
                dispatch_to_widget(interp, shared, *key);
            }
        }
    }

    // Check quit
    let quit = shared
        .lock()
        .map(|st| st.events.quit_requested)
        .unwrap_or(false);
    if quit {
        eval_handler(interp, shared, "quit_handler");
        return LoopControl::Quit;
    }

    // Render
    render_all(shared, ctx);

    LoopControl::Continue
}

/// Evaluate pending timer scripts.
fn eval_pending_timers(interp: &mut Interpreter, shared: &Arc<Mutex<SharedState>>) {
    let scripts: Vec<String> = {
        let Ok(mut st) = shared.lock() else { return };
        st.events
            .timers
            .drain(..)
            .filter(|t| t.delay_ms == 0)
            .map(|t| t.script)
            .collect()
    };
    for script in scripts {
        let _ = interp.eval(&script);
    }
}

/// Check global key bindings and evaluate matching script.
fn handle_key(interp: &mut Interpreter, shared: &Arc<Mutex<SharedState>>, key: KeyEvent) -> bool {
    let keyspec = key_to_spec(key);
    let script = {
        let Ok(st) = shared.lock() else { return false };
        st.events.key_bindings.get(&keyspec).cloned()
    };
    if let Some(script) = script {
        let _ = interp.eval(&script);
        return true;
    }
    false
}

/// Dispatch a key event to the focused widget.
fn dispatch_to_widget(interp: &mut Interpreter, shared: &Arc<Mutex<SharedState>>, key: KeyEvent) {
    let focused = {
        let Ok(st) = shared.lock() else { return };
        st.focused.clone()
    };
    let Some(widget_id) = focused else { return };

    let result = {
        let Ok(mut st) = shared.lock() else { return };
        match st.widgets.get_mut(&widget_id) {
            Some(entry) => entry.handle_key(key),
            None => return,
        }
    };

    match result {
        EventResult::Action(action) => {
            handle_widget_action(interp, shared, &widget_id, action);
        }
        EventResult::Consumed | EventResult::Ignored => {}
    }
}

/// Handle a widget action by calling the registered handler.
fn handle_widget_action(
    interp: &mut Interpreter,
    shared: &Arc<Mutex<SharedState>>,
    widget_id: &str,
    action: WidgetAction,
) {
    let (event_name, arg) = match action {
        WidgetAction::Selected(s) => ("on-select", s),
        WidgetAction::Confirmed(s) => ("on-submit", s),
        WidgetAction::Cancelled => ("on-cancel", String::new()),
        WidgetAction::Close => ("on-close", String::new()),
        WidgetAction::FocusNext | WidgetAction::FocusPrev => return,
        WidgetAction::Custom(_) => return,
    };

    let handler = {
        let Ok(st) = shared.lock() else { return };
        st.events
            .widget_handlers
            .get(&(widget_id.to_string(), event_name.to_string()))
            .cloned()
    };

    if let Some(proc_name) = handler {
        let call = if arg.is_empty() {
            proc_name
        } else {
            format!("{proc_name} {{{arg}}}")
        };
        let _ = interp.eval(&call);
    }
}

/// Evaluate a named handler (quit_handler or resize_handler).
fn eval_handler(interp: &mut Interpreter, shared: &Arc<Mutex<SharedState>>, handler_field: &str) {
    let script = {
        let Ok(st) = shared.lock() else { return };
        match handler_field {
            "quit_handler" => st.events.quit_handler.clone(),
            "resize_handler" => st.events.resize_handler.clone(),
            _ => None,
        }
    };
    if let Some(script) = script {
        let _ = interp.eval(&script);
    }
}

/// Render all widgets into their layout positions.
fn render_all(shared: &Arc<Mutex<SharedState>>, ctx: &mut RunContext) {
    let Ok(st) = shared.lock() else { return };
    let w = ctx.screen.width();
    let h = ctx.screen.height();
    let area = Rect { x: 0, y: 0, w, h };
    let positions = st.layout.compute(area);
    let focused_id = st.focused.as_deref();

    let mut surface = ctx.screen.full_surface();
    for (widget_id, rect) in &positions {
        if let Some(entry) = st.widgets.get(widget_id) {
            let is_focused = focused_id == Some(widget_id.as_str());
            let mut sub = surface.sub(rect.x, rect.y, rect.w, rect.h);
            entry.render(&mut sub, is_focused);
        }
    }
}

/// Convert a crossterm KeyEvent to a keyspec string like "Ctrl-Q".
fn key_to_spec(key: KeyEvent) -> String {
    let mut parts = Vec::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }
    let key_name = match key.code {
        KeyCode::Char(c) => {
            let upper = c.to_ascii_uppercase();
            return if parts.is_empty() {
                c.to_string()
            } else {
                format!("{}-{upper}", parts.join("-"))
            };
        }
        KeyCode::F(n) => format!("F{n}"),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Escape".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Delete => "Delete".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        _ => return String::new(),
    };
    if parts.is_empty() {
        key_name
    } else {
        format!("{}-{key_name}", parts.join("-"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_to_spec_simple_char() {
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(key_to_spec(key), "a");
    }

    #[test]
    fn key_to_spec_ctrl_q() {
        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(key_to_spec(key), "Ctrl-Q");
    }

    #[test]
    fn key_to_spec_f1() {
        let key = KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE);
        assert_eq!(key_to_spec(key), "F1");
    }

    #[test]
    fn key_to_spec_ctrl_shift_up() {
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert_eq!(key_to_spec(key), "Ctrl-Shift-Up");
    }

    #[test]
    fn key_to_spec_escape() {
        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(key_to_spec(key), "Escape");
    }

    #[test]
    fn event_state_default() {
        let es = EventState::new();
        assert!(es.key_bindings.is_empty());
        assert!(es.widget_handlers.is_empty());
        assert!(es.timers.is_empty());
        assert!(!es.quit_requested);
    }
}
