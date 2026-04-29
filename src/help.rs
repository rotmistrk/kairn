use crate::config::Config;

pub fn build_full_help(cfg: &Config) -> String {
    let mut h = String::new();
    help_header(&mut h);
    help_navigation(&mut h, cfg);
    help_panels(&mut h, cfg);
    help_operations(&mut h, cfg);
    help_config(&mut h, cfg);
    h
}

fn help_kb(cfg: &Config, name: &str) -> String {
    let key = cfg.display_key(name);
    let src = cfg.key_source(name).label();
    format!("`{key}` — {name} *({src})*")
}

fn help_header(h: &mut String) {
    h.push_str("# kairn v0.1.0\n\n");
    h.push_str("```\n  ╦╔═╔═╗╦╦═╗╔╗╔\n");
    h.push_str("  ╠╩╗╠═╣║╠╦╝║║║\n");
    h.push_str("  ╩ ╩╩ ╩╩╩╚═╝╚╝\n```\n\n");
    h.push_str("A TUI IDE for Kiro AI. Named after *cairn* — stacked stones marking a trail.\n\n");
    h.push_str("**Two-chord keys:** some bindings use a prefix (e.g. `Ctrl-X`) ");
    h.push_str("followed by a second key. The status bar shows the pending prefix.\n\n");
}

fn help_navigation(h: &mut String, cfg: &Config) {
    let kb = |n| help_kb(cfg, n);
    h.push_str("## Navigation\n\n");
    h.push_str("Three panels: **Tree ←→ Main ←→ Terminal**\n\n");
    h.push_str("**Spatial (arrow keys):**\n");
    h.push_str("- Tree: `→` on file → focus Main | `→` on dir → expand\n");
    h.push_str("- Main (scroll mode): `←` → Tree | `→` → Terminal\n");
    h.push_str("- Main (cursor mode): arrows move cursor within panel\n");
    h.push_str("- Terminal: `Esc Esc` or `Ctrl-]` → Main\n\n");
    h.push_str("**Direct focus:**\n");
    for name in ["focus_tree", "focus_main", "focus_terminal", "cycle_focus"] {
        h.push_str(&format!("- {}\n", kb(name)));
    }
    h.push('\n');
    h.push_str("**Layout:**\n");
    h.push_str(&format!("- {}\n", kb("rotate_layout")));
    h.push_str(&format!("- {}\n", kb("toggle_tree")));
    h.push_str(&format!(
        "- {} — toggle Files / Commits\n",
        kb("toggle_left_panel")
    ));
    h.push('\n');
    h.push_str("**Mode cycling** (`Ctrl-Shift-↑/↓` — context-aware):\n");
    h.push_str("- Tree focused: filter **All → Modified → Untracked**\n");
    h.push_str("- Main focused: view **File → Diff → Log → Blame**\n");
    h.push_str("- Terminal focused: switch tabs\n\n");
    h.push_str("**Resize:**\n");
    for name in [
        "resize_tree_shrink",
        "resize_tree_grow",
        "resize_interactive_shrink",
        "resize_interactive_grow",
    ] {
        h.push_str(&format!("- {}\n", kb(name)));
    }
    h.push_str("- Shift variants resize by 5\n");
    h.push_str("- In stacked layouts, F7/F8 resize terminal vertically when focused\n\n");
}

fn help_panels(h: &mut String, cfg: &Config) {
    let kb = |n| help_kb(cfg, n);
    h.push_str("## Main Panel\n\n");
    h.push_str("**Scroll mode** (default):\n");
    h.push_str("- `↑`/`↓`/`PgUp`/`PgDn` — scroll\n");
    h.push_str("- `←`/`→` — navigate to Tree / Terminal\n");
    h.push_str("- `/` — search as you type, `n`/`N` next/prev\n");
    h.push_str("- `Space` — enter cursor mode\n\n");
    h.push_str("**Cursor mode** (double-line border):\n");
    h.push_str("- `↑↓←→` — move cursor\n");
    h.push_str("- `v` stream / `V` line / `Ctrl-V` block select\n");
    h.push_str("- `Enter` — send selection to active terminal tab\n");
    h.push_str("- `Esc` — clear selection | `Space` — exit cursor mode\n\n");
    h.push_str("## File Tree\n\n");
    h.push_str("- `j`/`k` `↑`/`↓` — navigate (auto-preview in main)\n");
    h.push_str("- `Enter`/`l` — open file / expand dir\n");
    h.push_str("- `→` on file — focus main panel\n");
    h.push_str("- `h`/`←` — collapse dir (on leaf/collapsed: jump to parent)\n");
    h.push_str(&format!("- {} — refresh file tree\n", kb("refresh_tree")));
    h.push_str("- Git: **yellow**=modified **green**=added **red**=deleted\n\n");
    h.push_str("## Terminal Tabs\n\n");
    for name in ["new_kiro_tab", "new_shell_tab", "close_tab"] {
        h.push_str(&format!("- {}\n", kb(name)));
    }
    h.push_str("- `PgUp`/`PgDn` — scroll back\n");
    h.push_str("- `Ctrl-R` — rename tab\n");
    h.push_str("- `Ctrl-Enter` — expand @macros and send\n");
    h.push_str("- `Esc Esc` or `Ctrl-]` — escape to main panel\n\n");
    h.push_str("## Capture & Save\n\n");
    for name in ["capture_all", "capture_output", "save_buffer"] {
        h.push_str(&format!("- {}\n", kb(name)));
    }
    h.push_str("- capture_all scrapes the full terminal (scrollback + grid) into main\n");
    h.push_str("- capture_output extracts only the last command output\n");
    h.push_str("- save_buffer writes the current main panel content to a file\n\n");
}

fn help_operations(h: &mut String, cfg: &Config) {
    let kb = |n| help_kb(cfg, n);
    h.push_str("## File & Git Operations\n\n");
    for name in [
        "open_search",
        "launch_editor",
        "diff_current_file",
        "git_log",
        "show_help",
    ] {
        h.push_str(&format!("- {}\n", kb(name)));
    }
    h.push('\n');
    h.push_str("## Session & System\n\n");
    for name in [
        "save_session",
        "load_session",
        "suspend_to_shell",
        "peek_screen",
        "quit",
    ] {
        h.push_str(&format!("- {}\n", kb(name)));
    }
    h.push('\n');
    h.push_str("## Template Variables\n\n");
    h.push_str("Expand with `Ctrl-Enter` in terminal, or `Enter` from selection:\n\n");
    h.push_str("| Variable | Expands to |\n");
    h.push_str("|----------|------------|\n");
    h.push_str("| `@file` | Current file path |\n");
    h.push_str("| `@name` | Current file name |\n");
    h.push_str("| `@dir`  | Workspace root |\n");
    h.push_str("| `@line` | Cursor line number |\n\n");
}

fn help_config(h: &mut String, cfg: &Config) {
    h.push_str("## Configuration\n\n");
    h.push_str(&format!(
        "- **Global:** `{}`\n",
        Config::global_rc().display()
    ));
    h.push_str("- **Project:** `$PWD/.kairnrc` (overrides global)\n");
    h.push_str("- **State:** `$PWD/.kairn.state` (auto-saved on quit)\n\n");
    h.push_str("```json\n");
    h.push_str("{\n  \"kiro_command\": \"kiro-cli\",\n");
    h.push_str("  \"line_numbers\": true,\n");
    h.push_str("  \"keys\": { \"quit\": \"ctrl+q\" }\n}\n```\n\n");
    h.push_str("## Environment Variables\n\n");
    h.push_str("- `KAIRN_PID` — prevents nested instances\n");
    h.push_str("- `KAIRN_CAPTURE` — pipe: `cmd > $KAIRN_CAPTURE` → main panel\n");
    h.push_str("- `SHELL` — shell tabs | `EDITOR` — Ctrl-E\n\n");
    let conflicts = cfg.detect_collisions();
    if !conflicts.is_empty() {
        h.push_str("## ⚠ Key Conflicts\n\n");
        for c in &conflicts {
            h.push_str(&format!("- {c}\n"));
        }
        h.push('\n');
    }
}
