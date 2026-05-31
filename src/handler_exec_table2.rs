//! Dispatch table part 2 (N-Z).

use std::fs;

use txv_core::message::Message;
use txv_core::program::CommandContext;

use crate::commands::CM_ROOTS_CHANGED;
use crate::handler::AppState;
use crate::handler_exec::ExecEntry;

pub static TABLE_PART2: &[ExecEntry] = &[
    ExecEntry {
        names: &["move-tab"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_move_tab,
    },
    ExecEntry {
        names: &["new-dir"],
        requires_arg: true,
        handler: crate::handler_dired::cmd_new_dir,
    },
    ExecEntry {
        names: &["new-file"],
        requires_arg: true,
        handler: crate::handler_dired::cmd_new_file,
    },
    ExecEntry {
        names: &["next-error"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_next_error,
    },
    ExecEntry {
        names: &["noblame"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_noblame,
    },
    ExecEntry {
        names: &["paste"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_paste,
    },
    ExecEntry {
        names: &["prev-error"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_prev_error,
    },
    ExecEntry {
        names: &["quit"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_quit,
    },
    ExecEntry {
        names: &["rename-file"],
        requires_arg: true,
        handler: crate::handler_dired::cmd_rename_file,
    },
    ExecEntry {
        names: &["run"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_run,
    },
    ExecEntry {
        names: &["save"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_save,
    },
    ExecEntry {
        names: &["shell"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_shell,
    },
    ExecEntry {
        names: &["shrink"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_shrink,
    },
    ExecEntry {
        names: &["shrink-subpanel"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_shrink_subpanel,
    },
    ExecEntry {
        names: &["shrink-v"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_shrink_v,
    },
    ExecEntry {
        names: &["split"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_split,
    },
    ExecEntry {
        names: &["structured", "struct", "tree"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_structured,
    },
    ExecEntry {
        names: &["tab"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_tab,
    },
    ExecEntry {
        names: &["tab-next"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_tab_next,
    },
    ExecEntry {
        names: &["tab-prev"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_tab_prev,
    },
    ExecEntry {
        names: &["tab-rename"],
        requires_arg: true,
        handler: crate::handler_exec_nav::cmd_tab_rename,
    },
    ExecEntry {
        names: &["test"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_test,
    },
    ExecEntry {
        names: &["test-at-cursor"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_test_at_cursor,
    },
    ExecEntry {
        names: &["test-file"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_test_file,
    },
    ExecEntry {
        names: &["text"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_text,
    },
    ExecEntry {
        names: &["theme"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_theme,
    },
    ExecEntry {
        names: &["toggle-tools"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_toggle_tools,
    },
    ExecEntry {
        names: &["toggle-tree"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_toggle_tree,
    },
    ExecEntry {
        names: &["vsplit"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_vsplit,
    },
    ExecEntry {
        names: &["welcome"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_welcome,
    },
    ExecEntry {
        names: &["zoom"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_zoom,
    },
    ExecEntry {
        names: &["add-root"],
        requires_arg: true,
        handler: cmd_add_root,
    },
    ExecEntry {
        names: &["remove-root"],
        requires_arg: true,
        handler: cmd_remove_root,
    },
];

fn cmd_add_root(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    use std::path::PathBuf;
    let path = PathBuf::from(arg);
    let path = if path.is_relative() {
        state.root_dir().join(&path)
    } else {
        path
    };
    let Some(path) = fs::canonicalize(&path).ok() else {
        push_msg(state, Message::error("root", format!("Not found: {arg}")));
        return;
    };
    if !path.is_dir() {
        push_msg(state, Message::error("root", format!("Not a directory: {arg}")));
        return;
    }
    if !state.roots_mut().add(path.clone()) {
        push_msg(
            state,
            Message::warn("root", format!("Already a root: {}", path.display())),
        );
        return;
    }
    push_msg(state, Message::info("root", format!("Added root: {}", path.display())));
    refresh_completer_roots(state);
    emit_roots_changed(ctx, state);
}

fn cmd_remove_root(ctx: &mut CommandContext, state: &mut AppState, arg: &str) {
    use std::path::PathBuf;
    let path = PathBuf::from(arg);
    let path = if path.is_relative() {
        state.root_dir().join(&path)
    } else {
        path
    };
    let path = fs::canonicalize(&path).unwrap_or(path);
    if !state.roots_mut().remove(&path) {
        push_msg(
            state,
            Message::warn("root", format!("Not a root or last root: {}", path.display())),
        );
        return;
    }
    push_msg(
        state,
        Message::info("root", format!("Removed root: {}", path.display())),
    );
    refresh_completer_roots(state);
    emit_roots_changed(ctx, state);
}

pub(crate) fn refresh_completer_roots(state: &AppState) {
    let paths: Vec<String> = state.roots().paths().iter().map(|p| p.display().to_string()).collect();
    if let Ok(mut guard) = state.completer_roots.lock() {
        *guard = paths;
    }
}

fn emit_roots_changed(ctx: &mut CommandContext, state: &AppState) {
    use crate::commands::RootsChangedData;
    let data = RootsChangedData::from_roots(state.roots());
    ctx.sink.push_broadcast(CM_ROOTS_CHANGED, Some(Box::new(data)));
}

fn push_msg(state: &AppState, msg: Message) {
    if let Ok(mut ring) = state.messages().lock() {
        ring.push(msg);
    }
}
