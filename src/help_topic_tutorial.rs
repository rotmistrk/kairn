//! Help topic: quick-start tutorial.

fn section_nav() -> &'static str {
    "\
┌─ Navigation ─────────────────────────
│
│  Ctrl+Shift+←/→    Switch panes
│  Alt+1..9           Switch tabs
│  Alt+0              Tab dropdown
│  Alt+Shift+Arrows   Resize panels
│  F5                 Zoom/unzoom
│
└──────────────────────────────────────
"
}

fn section_commands() -> &'static str {
    "\
┌─ Commands ───────────────────────────
│
│  Alt+x (M-x)   Command line
│                 (Tab completes, Up history)
│  :e <file>     Open file
│  :w             Save
│  :q             Close tab
│  :help <topic>  Help
│  See :help commands for full list.
│
└──────────────────────────────────────
"
}

fn section_finder() -> &'static str {
    "\
┌─ File Finder ────────────────────────
│
│  Ctrl+P         Fuzzy file finder
│  Type fragments, Enter opens, Esc cancels.
│
└──────────────────────────────────────
"
}

fn section_shell() -> &'static str {
    "\
┌─ Shell & Kiro AI ────────────────────
│
│  F4             Open terminal
│  :kiro          Start AI agent
│  Todo panel (F2 → Todo tab) tracks tasks.
│
└──────────────────────────────────────
"
}

fn section_config() -> &'static str {
    "\
┌─ Configuration ──────────────────────
│
│  ~/.config/kairn/init.tcl  Global
│  .kairn/init.tcl           Project
│  See :help tcl for scripting.
│
└──────────────────────────────────────
"
}

fn section_roots() -> &'static str {
    "\
┌─ Multi-root Workspaces ─────────────
│
│  :add-root <path>    Add project root
│  Colored badges in tree and git.
│  :remove-root <path> to remove.
│
└──────────────────────────────────────
"
}

pub fn help_tutorial() -> String {
    let header = "\
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 QUICK START
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

";
    format!(
        "{}{}{}{}{}{}{}",
        header,
        section_nav(),
        section_commands(),
        section_finder(),
        section_shell(),
        section_config(),
        section_roots(),
    )
}
