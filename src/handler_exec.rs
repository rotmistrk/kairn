//! M-x command dispatch — handles CM_EXECUTE_COMMAND via dispatch table.

use txv_core::program::CommandContext;

use crate::handler::AppState;

/// A dispatch table entry. The table IS the command list.
pub struct ExecEntry {
    /// Command names (first is canonical, rest are aliases).
    pub(crate) names: &'static [&'static str],
    /// If true, the command requires a non-empty argument.
    pub(crate) requires_arg: bool,
    /// Handler function: receives context, state, and the argument string.
    pub(crate) handler: fn(&mut CommandContext, &mut AppState, &str),
}

/// The dispatch table — single source of truth for M-x commands.
/// The completer iterates this directly. No separate list to maintain.
pub fn dispatch_table() -> impl Iterator<Item = &'static ExecEntry> {
    crate::handler_exec_table1::TABLE_PART1
        .iter()
        .chain(crate::handler_exec_table2::TABLE_PART2.iter())
}

/// Handle the M-x command dispatch.
pub fn handle_execute_command(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(boxed) = ctx.data.as_ref() else {
        return;
    };
    let Some(text) = boxed.downcast_ref::<String>() else {
        return;
    };
    log::debug!("execute_command: {:?}", text);

    let parts: Vec<&str> = text.trim().splitn(2, ' ').collect();
    let cmd = parts.first().copied().unwrap_or("");
    let arg = parts.get(1).copied().unwrap_or("");

    for entry in dispatch_table() {
        if entry.names.contains(&cmd) {
            if entry.requires_arg && arg.is_empty() {
                return;
            }
            (entry.handler)(ctx, state, arg);
            return;
        }
    }

    execute_as_tcl(ctx, state, text);
}

fn execute_as_tcl(ctx: &mut CommandContext, state: &mut AppState, text: &str) {
    if is_bare_word(text) && !state.script.has_command(text) {
        let msg = txv_core::message::Message::error("cmd", format!("Unknown command: {text}"));
        ctx.sink
            .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    } else {
        match state.script.eval(text) {
            Ok(result) => {
                crate::completer::refresh_commands(&state.command_list, &state.script);
                let cmds = state.script.drain_commands();
                crate::handler_script::dispatch_script_commands(cmds, ctx, state);
                if !result.is_empty() {
                    let msg = txv_core::message::Message::info("tcl", result);
                    ctx.sink
                        .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
            }
            Err(e) => {
                let msg = txv_core::message::Message::error("tcl", e);
                ctx.sink
                    .push_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            }
        }
    }
}

/// A bare word is a single token with no Tcl syntax (no spaces, brackets, braces, quotes).
fn is_bare_word(s: &str) -> bool {
    !s.is_empty() && !s.contains(|c: char| c.is_whitespace() || "[]{}\"$;".contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::CM_EXECUTE_COMMAND;

    /// Verify every entry in the dispatch table is actually callable
    /// (not producing "Unknown command").
    #[test]
    fn dispatch_table_all_recognized() {
        use txv_core::event::Event;
        use txv_core::program::Program;

        let dir = std::env::temp_dir();
        let (mut program, sink, mut state) = setup_test_program(&dir);

        for entry in dispatch_table() {
            for &name in entry.names {
                verify_entry_recognized(name, entry, &mut program, &sink, &mut state);
            }
        }
    }

    fn setup_test_program(
        dir: &std::path::Path,
    ) -> (txv_core::program::Program, txv_core::prelude::EventSink, AppState) {
        let desktop = crate::build_desktop::build_workspace(dir, crate::settings::GitKeys::default());
        let state = crate::handler::AppState::new(dir.to_path_buf());
        let status = crate::status::build_status_bar(
            &desktop,
            Box::new(crate::completer::AppCompleter::new(
                dir.to_path_buf(),
                crate::completer::new_command_list(),
            )),
            0,
            dir.to_path_buf(),
            &crate::settings::StatusKeys::default(),
        );
        let mut program = txv_core::program::Program::new(Box::new(status), Box::new(desktop));
        let sink = program.sink().clone();
        (program, sink, state)
    }

    fn verify_entry_recognized(
        name: &str,
        entry: &ExecEntry,
        program: &mut txv_core::program::Program,
        sink: &txv_core::prelude::EventSink,
        state: &mut AppState,
    ) {
        use txv_core::event::Event;

        let text = if entry.requires_arg {
            format!("{name} test_arg")
        } else {
            name.to_string()
        };
        let data: Option<Box<dyn std::any::Any + Send>> = Some(Box::new(text));
        let mut ctx = txv_core::program::CommandContext {
            command: CM_EXECUTE_COMMAND,
            data: &data,
            sink,
            desktop: program.desktop_mut(),
        };
        handle_execute_command(&mut ctx, state);

        let events = sink.drain();
        let produced_unknown = events.iter().any(|ev| {
            if let Event::Command { id, data } = ev {
                if *id == txv_widgets::CM_STATUS_MESSAGE {
                    if let Some(msg) = data
                        .as_ref()
                        .and_then(|d| d.downcast_ref::<txv_core::message::Message>())
                    {
                        return msg.text.contains("Unknown command");
                    }
                }
            }
            false
        });
        assert!(
            !produced_unknown,
            "Dispatch table entry '{name}' produced 'Unknown command'. Bug in lookup logic."
        );
    }
}
