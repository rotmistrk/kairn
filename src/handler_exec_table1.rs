//! Dispatch table part 1 (A-M).

use crate::handler_exec::ExecEntry;

pub static TABLE_PART1: &[ExecEntry] = &[
    ExecEntry {
        names: &["blame"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_blame,
    },
    ExecEntry {
        names: &["build"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_build,
    },
    ExecEntry {
        names: &["close"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_close,
    },
    ExecEntry {
        names: &["code-action"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_code_action,
    },
    ExecEntry {
        names: &["copy-file"],
        requires_arg: true,
        handler: crate::handler_dired::cmd_copy_file,
    },
    ExecEntry {
        names: &["cycle-subpanel"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_cycle_subpanel,
    },
    ExecEntry {
        names: &["delete-file"],
        requires_arg: true,
        handler: crate::handler_dired::cmd_delete_file,
    },
    ExecEntry {
        names: &["diff"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_diff,
    },
    ExecEntry {
        names: &["edit", "e"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_edit,
    },
    ExecEntry {
        names: &["focus-down"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_focus_down,
    },
    ExecEntry {
        names: &["focus-left"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_focus_left,
    },
    ExecEntry {
        names: &["focus-right"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_focus_right,
    },
    ExecEntry {
        names: &["focus-up"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_focus_up,
    },
    ExecEntry {
        names: &["fmt!"],
        requires_arg: false,
        handler: crate::handler_format_builtin::cmd_format_builtin,
    },
    ExecEntry {
        names: &["git-base"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_git_base,
    },
    ExecEntry {
        names: &["git-commit"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_git_commit,
    },
    ExecEntry {
        names: &["git-stage"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_git_stage,
    },
    ExecEntry {
        names: &["git-unstage"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_git_unstage,
    },
    ExecEntry {
        names: &["git-untrack"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_git_untrack,
    },
    ExecEntry {
        names: &["grep"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_grep,
    },
    ExecEntry {
        names: &["replace", "sed"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_replace,
    },
    ExecEntry {
        names: &["grow"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_grow,
    },
    ExecEntry {
        names: &["grow-subpanel"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_grow_subpanel,
    },
    ExecEntry {
        names: &["grow-v"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_grow_v,
    },
    ExecEntry {
        names: &["help"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_help,
    },
    ExecEntry {
        names: &["kiro"],
        requires_arg: false,
        handler: crate::handler_kiro::cmd_kiro,
    },
    ExecEntry {
        names: &["layout"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_layout,
    },
    ExecEntry {
        names: &["log"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_log,
    },
    ExecEntry {
        names: &["lsp"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_lsp,
    },
    ExecEntry {
        names: &["lsp-rename"],
        requires_arg: true,
        handler: crate::handler_exec_edit::cmd_lsp_rename,
    },
    ExecEntry {
        names: &["lsp-status"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_lsp_status,
    },
    ExecEntry {
        names: &["messages"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_messages,
    },
    ExecEntry {
        names: &["clipboard"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_clipboard,
    },
    ExecEntry {
        names: &["problems"],
        requires_arg: false,
        handler: crate::handler_exec_edit::cmd_problems,
    },
    ExecEntry {
        names: &["set", "setglobal"],
        requires_arg: true,
        handler: crate::handler_exec_misc::cmd_set,
    },
];
