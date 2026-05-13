# kairn configuration — ~/.config/kairn/init.tcl
# Tcl syntax. Only set what you want to change; defaults apply for the rest.

# ─── Theme ───────────────────────────────────────────────────────────────────
# Mode: "auto" (detect from terminal), "dark", or "light"
set theme.mode "auto"

# ─── Editor ──────────────────────────────────────────────────────────────────
set editor.wrap false
set editor.list false
set editor.number true
set editor.tabstop 4

# ─── Terminal ────────────────────────────────────────────────────────────────
set terminal.scrollback 2000

# ─── Tabs ────────────────────────────────────────────────────────────────────
set tabs.max 10

# ─── Clock ───────────────────────────────────────────────────────────────────
# Status bar clock update interval in seconds (0 = disabled)
set clock.interval 60

# ─── Build / Run / Test ──────────────────────────────────────────────────────
# Auto-detected from Cargo.toml / Makefile / package.json if not set.
# set build.command "cargo build"
# set run.command "cargo run"
# set test.command "cargo test"

# ─── Git Panel Keys ──────────────────────────────────────────────────────────
set git.stage "s"
set git.unstage "u"
set git.untrack "x"
set git.commit "c"

# ─── Status Bar Keys (visible labels) ────────────────────────────────────────
set keys.help "F1"
set keys.tree "F2"
set keys.main "F3"
set keys.term "F4"
set keys.zoom "F5"
set keys.messages "F6"
set keys.quit "ctrl+q"

# ─── Colors ──────────────────────────────────────────────────────────────────
# Colors use ANSI 256 numbers (0-255) or "reset" for terminal default.
# Format: set color.<role> <ansi-number>
#
# Framework palette (txv-core):
#   base:        text, dim, bright, border, separator
#   interactive: cursor_focused, cursor_unfocused, input_cursor,
#                edit_overlay, search_match, visual_selection, disabled
#   chrome:      bar, tab_focused, tab_focused_arrow, tab_focused_badge,
#                tab_active, tab_active_arrow, tab_active_badge,
#                status_bar, scrollbar_track, scrollbar_thumb
#   popup:       background, border, selected, table_header
#   state:       error, warning, info, success, hint
#
# Application palette (kairn):
#   git:    added, modified, untracked, ignored, conflict
#   diff:   added, deleted, fold
#   editor: gutter, list_chars, cursor
#   diag:   error, warning, info, hint
#   tree:   directory
#   todo:   normal, done, important
#   msg:    error, warning, info, debug
#
# Example overrides (uncomment to customize):
# set color.base.dim 8
# set color.chrome.bar.fg 7
# set color.chrome.bar.bg 0
# set color.git.added 2
# set color.git.modified 12
# set color.git.untracked 1
# set color.diff.added 2
# set color.diff.deleted 1
# set color.editor.gutter 8
# set color.diag.error 1
# set color.tree.directory 14
# set color.todo.done 8
# set color.todo.important 1
# set color.state.error 1
# set color.state.warning 3
# set color.state.success 2

# ─── LSP ─────────────────────────────────────────────────────────────────────
# LSP servers are auto-detected. Override with:
# lsp rust-analyzer {
#     command "rust-analyzer"
#     filetypes {rs}
# }
# lsp typescript-language-server {
#     command "typescript-language-server --stdio"
#     filetypes {ts tsx js jsx}
# }
