//! TerminalView — real PTY terminal backed by txv-widgets::PtyTerminal.

pub use txv_widgets::PtyTerminal as TerminalView;

/// Create a shell terminal, falling back to a placeholder on failure.
pub fn new_shell_terminal() -> Box<dyn txv_core::view::View> {
    new_shell_terminal_with_scrollback(2000)
}

/// Create a shell terminal with custom scrollback, falling back to a placeholder on failure.
pub fn new_shell_terminal_with_scrollback(scrollback_lines: u16) -> Box<dyn txv_core::view::View> {
    // In test environments, don't spawn a real PTY
    if std::env::var("KAIRN_TEST").is_ok() {
        return Box::new(FallbackTerminal::new("Shell"));
    }
    match txv_widgets::PtyTerminal::spawn_shell_with_scrollback(80, 24, scrollback_lines as usize) {
        Ok(term) => Box::new(term),
        Err(e) => {
            log::error!("Failed to spawn shell: {}", e);
            Box::new(FallbackTerminal::with_error("Shell (failed)", format!("{e}")))
        }
    }
}

/// Create a shell terminal that runs a specific command.
pub fn new_shell_with_command(cmd: &str, cwd: &std::path::Path) -> Box<dyn txv_core::view::View> {
    if std::env::var("KAIRN_TEST").is_ok() {
        return Box::new(FallbackTerminal::new("Run"));
    }
    match txv_widgets::PtyTerminal::spawn_command("sh", &["-c", cmd], cwd, 80, 24) {
        Ok(term) => Box::new(term),
        Err(e) => {
            log::error!("Failed to run command '{}': {}", cmd, e);
            Box::new(FallbackTerminal::with_error("Run (failed)", format!("{e}")))
        }
    }
}

/// Create a kiro-cli chat terminal, optionally with a specific agent.
pub fn new_kiro_terminal(agent: Option<&str>, cwd: &std::path::Path) -> Box<dyn txv_core::view::View> {
    new_kiro_terminal_with_resume(agent, None, cwd)
}

/// Create a kiro-cli chat terminal with optional agent and resume-id.
pub fn new_kiro_terminal_with_resume(
    agent: Option<&str>,
    resume_id: Option<&str>,
    cwd: &std::path::Path,
) -> Box<dyn txv_core::view::View> {
    if std::env::var("KAIRN_TEST").is_ok() {
        return Box::new(FallbackTerminal::new("Kiro"));
    }
    let mut args = vec!["chat"];
    let agent_flag;
    if let Some(name) = agent {
        agent_flag = format!("--agent={name}");
        args.push(&agent_flag);
    }
    let resume_flag;
    if let Some(id) = resume_id {
        resume_flag = format!("--resume-id={id}");
        args.push(&resume_flag);
    }
    let socket_val = std::env::var("KAIRN_MCP_SOCKET").unwrap_or_default();
    let envs: Vec<(&str, &str)> = if socket_val.is_empty() {
        vec![]
    } else {
        vec![("KAIRN_MCP_SOCKET", &socket_val)]
    };
    match txv_widgets::PtyTerminal::spawn_command_with_env("kiro-cli", &args, cwd, 80, 24, &envs) {
        Ok(term) => Box::new(term),
        Err(e) => {
            log::error!("Failed to spawn kiro: {}", e);
            Box::new(FallbackTerminal::with_error("Kiro (failed)", format!("kiro-cli: {e}")))
        }
    }
}

/// Minimal fallback when PTY spawn fails.
struct FallbackTerminal {
    state: txv_core::prelude::ViewState,
    title: String,
    message: String,
}

impl FallbackTerminal {
    fn new(title: impl Into<String>) -> Self {
        Self {
            state: txv_core::prelude::ViewState::default(),
            title: title.into(),
            message: String::new(),
        }
    }

    fn with_error(title: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            state: txv_core::prelude::ViewState::default(),
            title: title.into(),
            message: error.into(),
        }
    }
}

impl txv_core::view::View for FallbackTerminal {
    txv_core::delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        &self.title
    }

    fn draw(&mut self) {
        let style = txv_core::cell::Style::default();
        let err_style = txv_core::palette::palette().state().error();
        self.state.buffer_mut().print(0, 0, &format!("[{}]", self.title), style);
        if !self.message.is_empty() {
            self.state.buffer_mut().print(0, 1, &self.message, err_style);
            self.state
                .buffer_mut()
                .print(0, 2, "Check that the command is installed and in PATH.", style);
        }
    }

    fn handle(&mut self, _event: &txv_core::event::Event) -> txv_core::view::HandleResult {
        txv_core::view::HandleResult::Ignored
    }
}
