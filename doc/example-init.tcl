# kairn configuration — ~/.config/kairn/init.tcl
# Tcl syntax. Only set what you want to change; defaults apply for the rest.

# ─── Theme ───────────────────────────────────────────────────────────────────
# Mode: "auto" (detect from terminal), "dark", or "light"
set theme.mode "auto"

# Syntax highlighting theme (syntect theme names)
# set theme.syntax_dark "base16-eighties.dark"
# set theme.syntax_light "base16-ocean.light"

# Glyph style: "auto" (detect from terminal), "ascii", "utf", or "nerd"
# set theme.glyphs "auto"

# ─── Editor ──────────────────────────────────────────────────────────────────
set editor.wrap false
set editor.list false
set editor.number true
set editor.tabstop 4

# Cursor shapes per mode: bar, block, underline, software, none
set editor.cursor_insert bar
set editor.cursor_normal software
set editor.cursor_command software

# ─── Terminal ────────────────────────────────────────────────────────────────
set terminal.scrollback 2000
# Shell tabs auto-update their title from the terminal's OSC title
# (set by shell prompt hooks, e.g. \e]0;~/projects\a). Format: "Shell:<title>"

# ─── Layout ──────────────────────────────────────────────────────────────────
# Auto-switch thresholds (terminal width in columns):
#   wide-threshold: switch from tall to wide when width >= this (default: 300)
#   tall-threshold: switch from wide to tall when width <= this (default: 200)
# set layout.wide-threshold 300
# set layout.tall-threshold 200

# ─── Tabs ────────────────────────────────────────────────────────────────────
set tabs.max 10

# ─── Clock ───────────────────────────────────────────────────────────────────
# Status bar clock update interval in seconds (0 = disabled)
set clock.interval 60

# ─── Build / Run / Test ──────────────────────────────────────────────────────
# Build commands are auto-detected from Cargo.toml / Makefile / package.json.
# Override in .kairn/init (plain text, not Tcl):
#   build = make -j8
#   test = make check
#   test-file = cargo test --lib {file}
#
# Tcl override: define a proc to replace the auto-detected command.
# If the proc returns non-empty, it replaces the default. Return "" to fall back.
#
# proc build-command {} { return "make -j8" }
# proc test-command {} { return "cargo test --workspace" }
# proc run-command {} { return "./target/debug/myapp" }
#
# Project-specific overrides go in .kairn/init.tcl:
# proc build-command {} {
#     set file [editor current-file]
#     if {[string match "*.go" $file]} { return "go build ./..." }
#     return ""  ;# fall back to auto-detect
# }

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

# ─── Subpanel Keys (split pane navigation) ───────────────────────────────────
set keys.subpanel_focus "Ctrl-w"
set keys.subpanel_move "Ctrl-Alt-w"
set keys.subpanel_grow "Ctrl-Alt-="
set keys.subpanel_shrink "Ctrl-Alt--"

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
# set lsp.timeout 10          ;# seconds to wait for LSP response (default: 10)
# LSP servers are auto-detected. Override with:
# lsp args rust rust-analyzer --target-dir /tmp/target
# lsp args python pyright --stdio
#
# Set environment variables for LSP servers:
# lsp env rust CARGO_TARGET_DIR "/tmp/kairn-target"
# lsp env rust RUST_BACKTRACE 1
#
# Use lsp-start hook for dynamic configuration:
# hook add lsp-start -filter "rust" {
#     set target [system exec "cargo metadata --format-version=1 2>/dev/null | jq -r .target_directory"]
#     if {$target ne ""} { lsp env rust CARGO_TARGET_DIR $target }
# }

# ─── Hooks & Selection Scripting ─────────────────────────────────────────────

# ─── View Commands ───────────────────────────────────────────────────────────
# view theme dark|light|auto|toggle
# view zoom                    ;# toggle maximize current panel
# view toggle-tree             ;# show/hide file tree
# view toggle-tools            ;# show/hide tools panel
# view layout                  ;# cycle layout mode (auto/wide/tall)
# view focus left|center|right ;# focus a panel
# view status "text"           ;# flash text in status bar
# view message info|warn|error origin "text"

# ─── Build Commands ──────────────────────────────────────────────────────────
# build run ?<cmd>?            ;# run build (optional custom command)
# build test ?<cmd>?           ;# run tests
# build test-file              ;# test current file
# build test-at-cursor         ;# test function at cursor
# build next-error             ;# jump to next error
# build prev-error             ;# jump to previous error

# ─── Git Commands ────────────────────────────────────────────────────────────
# git stage <file>             ;# stage a file
# git unstage <file>           ;# unstage a file
# git untrack <file>           ;# untrack a file
# git commit <message>         ;# commit with message
# git blame                    ;# show blame annotations
# git noblame                  ;# hide blame annotations
# git log                      ;# show git log
# git diff                     ;# show diff for current file

# ─── Split Commands ──────────────────────────────────────────────────────────
# split vsplit ?<file>?        ;# vertical split
# split hsplit ?<file>?        ;# horizontal split
# split close                  ;# close split
# split focus                  ;# cycle focus between panes
# split open <path>            ;# open file in other pane
# split linked ?<bool>?        ;# toggle linked scroll
# split direction              ;# query current split direction

# ─── Editor Commands ─────────────────────────────────────────────────────────
# editor open <path> ?-line N? ?-col N?
# editor save / save-all / close
# editor undo / redo
# editor goto <line> ?<col>?
# editor insert <text>
# editor search <pattern>      ;# highlight matches
# editor clear-highlight       ;# clear search highlight
# editor get-selection         ;# returns selected text
# editor replace-selection <t> ;# replace selection
# editor get-line ?<n>?        ;# get line text (default: cursor line)
# editor delete-line ?<n>?     ;# delete line
# editor replace-word <text>   ;# replace word under cursor
# editor diff-revert           ;# revert diff hunk at cursor
# editor current-file / current-line / current-col
# editor modified? / filetype

# ─── LSP Commands ────────────────────────────────────────────────────────────
# lsp hover                    ;# show hover info
# lsp definition               ;# go to definition
# lsp references               ;# find references
# lsp rename <new-name>        ;# rename symbol
# lsp format                   ;# format document
# lsp start ?<pattern>?        ;# start LSP server matching pattern
# lsp restart ?<pattern>?      ;# restart LSP server
# lsp stop ?<pattern>?         ;# stop LSP server
# lsp status                   ;# show LSP status (via M-x lsp-status)

# ─── Todo Commands ───────────────────────────────────────────────────────────
# todo add <text> ?-parent <path>?  ;# add item (path = dot-separated indices)
# todo remove <path>           ;# remove item
# todo complete <path>         ;# toggle completion
# todo toggle-important <path> ;# toggle important flag
# todo edit <path> <text>      ;# rename item
# todo swap <path> up|down     ;# reorder item
# todo promote <path>          ;# decrease nesting
# todo demote <path>           ;# increase nesting
# todo list                    ;# (reserved, tree panel shows todos)

# ─── Grep ────────────────────────────────────────────────────────────────────
# grep <pattern>               ;# search project files, open results tab

# ─── System Commands ─────────────────────────────────────────────────────────
# system exec <cmd>            ;# run shell command, return output
# system env <var>             ;# get environment variable
# system set-env <var> <val>   ;# set environment variable
# system root-dir              ;# project root directory
# system home-dir              ;# user home directory
# system platform              ;# os name
# system clipboard-get         ;# read system clipboard
# system clipboard-set <text>  ;# write to system clipboard

# ─── Project Root Override ───────────────────────────────────────────────────
# Define a `project-root` proc to override automatic root detection.
# Called with the file path when opening a file via CLI.
# Return a directory path, or empty string to fall back to built-in detection.
#
# Example: super-project with multiple git repos under one workspace:
# proc project-root {path} {
#     # Always use ~/workspace/myproject as root
#     if {[string match "*/myproject/*" $path]} {
#         return "/home/user/workspace/myproject"
#     }
#     return ""
# }

# ─── Hooks & Key Bindings ────────────────────────────────────────────────────
# Auto-close brackets via char-inserted hook with filter:
# hook add char-inserted -filter "(" { editor insert ")" }
# hook add char-inserted -filter "{" { editor insert "}" }
# hook add char-inserted -filter "[" { editor insert "]" }

# Quote the current selection:
# keymap bind ctrl+q {
#   set sel [editor get-selection]
#   editor replace-selection "\"$sel\""
# }

# Word expansion via word-completed hook:
# hook add word-completed -filter "todo" {
#   editor replace-word "// TODO(user): "
# }

# Format on idle (fires after no keystrokes for idle timeout):
# hook add idle { lsp format }
