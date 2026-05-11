//! Diff command handler — parses flags, generates unified diff, displays result.

use txv_core::cell::Color;
use txv_core::program::CommandContext;
use txv_widgets::TextArea;

use crate::desktop::SlotId;
use crate::diff::{git_file_content, unified_diff, DiffOptions};
use crate::handler::{downcast_desktop, AppState};

/// Parsed diff command arguments.
struct DiffArgs {
    context: usize,
    ignore_whitespace: bool,
    base: String,
}

fn parse_diff_args(args: &str) -> DiffArgs {
    let mut result = DiffArgs {
        context: 3,
        ignore_whitespace: false,
        base: "HEAD".to_string(),
    };
    let mut iter = args.split_whitespace().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-w" {
            result.ignore_whitespace = true;
        } else if arg == "--base" {
            if let Some(val) = iter.next() {
                result.base = val.to_string();
            }
        } else if let Some(n) = arg.strip_prefix("-U") {
            if let Ok(ctx) = n.parse::<usize>() {
                result.context = ctx;
            }
        }
    }
    result
}

pub fn handle_diff(ctx: &mut CommandContext, state: &AppState) {
    let args_str = ctx
        .data
        .as_ref()
        .and_then(|b| b.downcast_ref::<String>())
        .map(|s| s.as_str())
        .unwrap_or("");
    let args = parse_diff_args(args_str);

    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(title) = desktop.active_tab_title(SlotId::Center).map(|s| s.to_string()) else {
        show_error(ctx, "No file open");
        return;
    };

    let rel_path = title.clone();
    let file_path = state.root_dir.join(&rel_path);

    // Read current file from disk
    let current = match std::fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            show_error(ctx, &format!("read: {e}"));
            return;
        }
    };

    // Get base content from git
    let base_content = match git_file_content(&state.root_dir, &rel_path, &args.base) {
        Ok(c) => c,
        Err(e) => {
            show_error(ctx, &format!("diff: {e}"));
            return;
        }
    };

    let opts = DiffOptions {
        context: args.context,
        ignore_whitespace: args.ignore_whitespace,
    };
    let old_label = format!("a/{rel_path} ({})", args.base);
    let new_label = format!("b/{rel_path} (working)");
    let diff_lines = unified_diff(&base_content, &current, &old_label, &new_label, &opts);

    if diff_lines.len() <= 2 {
        show_error(ctx, "No differences");
        return;
    }

    // Build colored TextArea
    let mut ta = TextArea::new();
    ta.line_numbers = false;
    let text: String = diff_lines.iter().map(|(_, l)| format!("{l}\n")).collect();
    ta.set_content(&text);
    ta.line_colors = diff_lines
        .iter()
        .map(|(tag, _)| match tag {
            '+' => Color::Ansi(2),
            '-' => Color::Ansi(1),
            '@' => Color::Ansi(6),
            _ => Color::Ansi(7),
        })
        .collect();

    let tab_title = format!("[diff] {rel_path}");
    let desktop = downcast_desktop(ctx.desktop).unwrap();
    desktop.insert_tab(SlotId::Center, tab_title, Box::new(ta));
}

fn show_error(ctx: &mut CommandContext, msg: &str) {
    ctx.queue
        .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg.to_string())));
}
