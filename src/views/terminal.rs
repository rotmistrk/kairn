//! TerminalView — real PTY terminal backed by txv-widgets::PtyTerminal.

pub use txv_widgets::PtyTerminal as TerminalView;

/// Create a shell terminal, falling back to a placeholder on failure.
pub fn new_shell_terminal() -> Box<dyn txv_core::view::View> {
    // In test environments, don't spawn a real PTY
    if std::env::var("KAIRN_TEST").is_ok() {
        return Box::new(FallbackTerminal::new("Shell"));
    }
    match txv_widgets::PtyTerminal::spawn_shell(80, 24) {
        Ok(term) => Box::new(term),
        Err(e) => {
            log::error!("Failed to spawn shell: {}", e);
            Box::new(FallbackTerminal::new("Shell (failed)"))
        }
    }
}

/// Create a kiro-cli chat terminal, optionally with a specific agent.
pub fn new_kiro_terminal(agent: Option<&str>) -> Box<dyn txv_core::view::View> {
    if std::env::var("KAIRN_TEST").is_ok() {
        return Box::new(FallbackTerminal::new("Kiro"));
    }
    let mut args = vec!["chat"];
    let agent_flag;
    if let Some(name) = agent {
        agent_flag = format!("--agent={name}");
        args.push(&agent_flag);
    }
    match txv_widgets::PtyTerminal::spawn_command(
        "kiro-cli",
        &args,
        &std::env::current_dir().unwrap_or_default(),
        80,
        24,
    ) {
        Ok(term) => Box::new(term),
        Err(e) => {
            log::error!("Failed to spawn kiro: {}", e);
            Box::new(FallbackTerminal::new("Kiro (failed)"))
        }
    }
}

/// Minimal fallback when PTY spawn fails.
struct FallbackTerminal {
    state: txv_core::prelude::ViewState,
    title: String,
}

impl FallbackTerminal {
    fn new(title: impl Into<String>) -> Self {
        Self {
            state: txv_core::prelude::ViewState::default(),
            title: title.into(),
        }
    }
}

impl txv_core::view::View for FallbackTerminal {
    txv_core::delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        &self.title
    }

    fn draw(&self, surface: &mut txv_core::surface::Surface) {
        let b = self.state.bounds;
        let style = txv_core::cell::Style::default();
        surface.print(b.x, b.y, &format!("[{}]", self.title), style);
    }

    fn handle(
        &mut self,
        _event: &txv_core::event::Event,
        _queue: &mut txv_core::view::EventQueue,
    ) -> txv_core::view::HandleResult {
        txv_core::view::HandleResult::Ignored
    }
}
