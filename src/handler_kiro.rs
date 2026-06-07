//! Kiro session launch handler — builds argv from settings, patches agent, spawns terminal.

use txv_core::message::Message;
use txv_core::program::CommandContext;

use crate::desktop::{next_tab_name, SlotId};
use crate::handler::{downcast_desktop, AppState};
use crate::handler_evict::try_insert_tab;
use crate::mcp::agent_patch::ensure_agent_patched;
use crate::views::terminal::new_kiro_terminal_argv;

pub(crate) fn cmd_kiro(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    let sink = ctx.sink().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let extra_args = shell_words(arg);
    let agent_name = extract_agent_name(&extra_args).unwrap_or("kairn");

    let patched_agent = match ensure_agent_patched(&state.root_dir, agent_name) {
        Ok(name) => name,
        Err(e) => {
            sink.push_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::error("kiro", e))),
            );
            return;
        }
    };

    let argv = build_kiro_argv(&state.settings.kiro().cmd, &patched_agent, &extra_args);
    let name = next_tab_name(desktop, SlotId::Tools, "Kiro");
    let term = new_kiro_terminal_argv(&argv, &state.root_dir);
    try_insert_tab(desktop, state, &sink, SlotId::Tools, name.clone(), term);
    state.kiro_registry.register(&name);
    sink.push_command(
        txv_widgets::CM_STATUS_MESSAGE,
        Some(Box::new(Message::info("kiro", format!("Started: {name}")))),
    );
}

fn build_kiro_argv(base_cmd: &[String], agent: &str, extra_args: &[&str]) -> Vec<String> {
    let mut argv = base_cmd.to_vec();
    if !argv.iter().any(|a| a.starts_with("--agent")) {
        argv.push(format!("--agent={agent}"));
    }
    argv.extend(
        extra_args
            .iter()
            .filter(|a| !a.starts_with("--agent"))
            .map(|s| s.to_string()),
    );
    argv
}

/// Extract agent name from args like ["--agent=foo"] or ["--agent", "foo"].
fn extract_agent_name<'a>(args: &'a [&str]) -> Option<&'a str> {
    for (i, arg) in args.iter().enumerate() {
        if let Some(name) = arg.strip_prefix("--agent=") {
            return Some(name);
        }
        if *arg == "--agent" {
            return args.get(i + 1).copied();
        }
    }
    None
}

/// Simple word splitting on whitespace.
fn shell_words(s: &str) -> Vec<&str> {
    s.split_whitespace().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_agent_name_equals_form() {
        let args = vec!["--agent=foo"];
        assert_eq!(extract_agent_name(&args), Some("foo"));
    }

    #[test]
    fn extract_agent_name_separate_form() {
        let args = vec!["--agent", "bar"];
        assert_eq!(extract_agent_name(&args), Some("bar"));
    }

    #[test]
    fn extract_agent_name_none_when_absent() {
        let args = vec!["--resume", "--tui"];
        assert_eq!(extract_agent_name(&args), None);
    }

    #[test]
    fn extract_agent_name_separate_form_missing_value() {
        let args = vec!["--agent"];
        assert_eq!(extract_agent_name(&args), None);
    }

    #[test]
    fn shell_words_splits_on_whitespace() {
        let result = shell_words("--agent=foo --resume");
        assert_eq!(result, vec!["--agent=foo", "--resume"]);
    }

    #[test]
    fn shell_words_empty_string() {
        let result = shell_words("");
        assert!(result.is_empty());
    }

    #[test]
    fn shell_words_extra_whitespace() {
        let result = shell_words("  --tui   --agent=x  ");
        assert_eq!(result, vec!["--tui", "--agent=x"]);
    }
}
