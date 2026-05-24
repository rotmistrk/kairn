//! Dispatch table part 2 (N-Z).

use crate::handler_exec::ExecEntry;

pub static TABLE_PART2: &[ExecEntry] = &[
    ExecEntry {
        names: &["move-tab"],
        requires_arg: false,
        handler: crate::handler_exec_nav::cmd_move_tab,
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
];
