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
# set editor.rainbow true       ;# colored bracket pairs (default: true)
# set editor.guides true        ;# indent guide lines (default: true)

# ─── File Tree ───────────────────────────────────────────────────────────────
# File/dir icons require a Nerd Font (https://www.nerdfonts.com/).
# set tree.icons true            ;# show Nerd Font icons in file tree (default: false)

# Cursor shapes per mode: bar, block, underline, software, none
set editor.cursor_insert bar
set editor.cursor_normal software
set editor.cursor_command software

# NOTE: editor.autosave and editor.autosave_delay are not yet configurable via Tcl.


# ─── Terminal ────────────────────────────────────────────────────────────────
set terminal.scrollback 2000
# Seconds before terminal is considered idle (default: 3)
# set terminal.idle-timeout 3
# Auto-close shell tabs when the shell process exits (default: true)
# set terminal.auto-close-on-exit true

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

# ─── Fixed Workspace Keys (not configurable) ─────────────────────────────────
# Alt-w             Close active tab
# Alt-,             Toggle tree panel
# Alt-.             Toggle tools panel
# Alt-/             Zoom toggle
# Alt-\             Cycle layout (auto/wide/tall)
# Alt-;             Next tab
# Alt-'             Previous tab
# Alt-0             Tab dropdown
# Alt-1..9          Select tab by number
# Alt-Shift-Arrows  Resize panels
# M-x (Alt-x)      Command line (with Tab completion and Up/Down history)

# ─── Colors ──────────────────────────────────────────────────────────────────
# Colors use ANSI 256 numbers (0-255) or "reset" for terminal default.
# Format: set color.<role> <ansi-number>
#
# Chrome/framework styles (configurable via "fg bg [attrs]" format):
#   chrome:      status_bar, status_bar_modal, bar, tab_focused, tab_active,
#                scrollbar_track, scrollbar_thumb, status_question, status_highlight
#   popup:       background, border, selected
#   interactive: cursor_focused, input_cursor, search_match
#
#   fg/bg: ansi number (0-15), p:N (palette 0-255), rgb:RRGGBB
#   attrs: bold, italic, underline, dim (space-separated)
#
# Example overrides (uncomment to customize):
# set color.chrome.status_bar       "7 p:236"
# set color.chrome.status_bar_modal "15 p:18"
# set color.chrome.bar              "7 0"
# set color.chrome.tab_focused      "14 4 bold"
# set color.chrome.tab_active       "0 rgb:c0c0c0 bold"
# set color.chrome.scrollbar_track  "8"
# set color.chrome.scrollbar_thumb  "0 7"
# set color.popup.background        "15 0"
# set color.popup.border            "6 0"
# set color.popup.selected          "15 4 underline"
# set color.interactive.cursor_focused  "0 4 underline"
# set color.interactive.input_cursor    "0 7"
# set color.interactive.search_match    "0 3"
#
# App-level colors (fg only, ansi number):
#   git:    added, modified, untracked, ignored, conflict
#   diff:   added, deleted, fold
#   editor: gutter, list_chars
#   diag:   error, warning, info, hint
#   tree:   directory
#   todo:   normal, done, important
#   msg:    error, warning, info, debug
#
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

# ─── LSP ─────────────────────────────────────────────────────────────────────
# set lsp.timeout 10          ;# seconds to wait for LSP response (default: 10)
#
# LSP servers are auto-detected. Override with lsp-server command:
# lsp-server rust rust-analyzer --target-dir /tmp/target
# lsp-server python pyright --stdio
# lsp-server typescript typescript-language-server --stdio
#
# Disable LSP for a language:
# lsp-disable markdown
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

# ─── Hooks ───────────────────────────────────────────────────────────────────
# Available hook events:
#   file-save          Fires after a file is saved
#   file-open          Fires when a file is opened
#   file-close         Fires when a file tab is closed
#   build-done         Fires when a build/test command completes
#   tab-switched       Fires when the active tab changes
#   startup            Fires once at application startup
#   char-inserted      Fires after a character is typed (filter = the char)
#   char-deleted       Fires after a character is deleted (filter = the char)
#   word-completed     Fires when a word boundary is reached (filter = the word)
#   idle               Fires after no keystrokes for idle timeout (filter = ms)
#   paste              Fires after text is pasted
#   mode-changed       Fires when editor mode changes (filter = mode name)
#   selection-changed  Fires when editor selection changes
#   lsp-start          Fires when an LSP server starts (filter = language)
#
# Syntax: hook add <event> ?-filter <pattern>? { body }
#
# Examples:
# hook add file-save { build run }
# hook add file-open -filter "*.rs" { lsp start }
# hook add build-done { view message info build "Build finished" }
# hook add startup { view status "Welcome!" }
# hook add char-inserted -filter "(" { editor insert ")" }
# hook add char-inserted -filter "{" { editor insert "}" }
# hook add char-deleted -filter "(" { }
# hook add word-completed -filter "todo" { editor replace-word "// TODO(user): " }
# hook add idle { lsp format }
# hook add paste { view status "Pasted" }
# hook add mode-changed -filter "insert" { view status "-- INSERT --" }
# hook add selection-changed { }
# hook add tab-switched { view status [editor current-file] }

# ─── Key Bindings ────────────────────────────────────────────────────────────
# Quote the current selection:
# keymap bind ctrl+q {
#   set sel [editor get-selection]
#   editor replace-selection "\"$sel\""
# }

# ─── File Operations (Alt-f / ƒ prefix in tree) ──────────────────────────────
# Alt-f (ƒ on macOS) opens file ops prefix in tree panel:
#   n:new  N:dir  d:del  r:rename  c:copy  m:mark  u:unmark  M:Move  C:Copy
# These commands also work from M-x:
#   new-file <path>         ;# create file (and parent dirs)
#   new-dir <path>          ;# create directory
#   delete-file <path>      ;# delete file or directory
#   rename-file <old> <new> ;# rename/move
#   copy-file <src> <dest>  ;# copy file or directory

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
# grep -a <pattern>            ;# search all workspace roots

# ─── System Commands ─────────────────────────────────────────────────────────
# system exec <cmd>            ;# run shell command, return output
# system env <var>             ;# get environment variable
# system set-env <var> <val>   ;# set environment variable
# system root-dir              ;# project root directory
# system roots                 ;# all workspace roots (newline-separated)
# system add-root <path>       ;# add workspace root directory
# system remove-root <path>    ;# remove workspace root
# system home-dir              ;# user home directory
# system platform              ;# os name
# system clipboard-get         ;# read system clipboard
# system clipboard-set <text>  ;# write to system clipboard

# ─── Window Title ────────────────────────────────────────────────────────────
# Tcl expression evaluated periodically. Result becomes the terminal window title.
# OSC 2 is emitted only when the result changes.
#
# Available commands for use in the expression:
#   system user              — current username
#   system hostname          — full hostname
#   system hostname 1        — first component of hostname (before first '.')
#   system short-pwd 20      — project root, ~ for home, smart-truncated to 20 chars
#   system busy              — returns "*" if kiro sessions are active, "" otherwise
#
set window.title-expr {kairn:[system user]@[system hostname 1]:[system short-pwd 20][system busy]}

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

# ─── Navigation ──────────────────────────────────────────────────────────────
# Ctrl+P           Fuzzy file finder (type path fragments, Enter opens)
# /                Incremental search (cursor jumps to match as you type)
#                  Backspace returns cursor toward start, Esc cancels
#
# Incremental search is enabled by default. To disable:
# set editor.incsearch false
#
# Scrolloff — keep cursor N lines from viewport edge (default: 3):
# set editor.scrolloff 5
#
# ─── CSV/Table View ──────────────────────────────────────────────────────────
# Left/Right arrows  Horizontal scroll (columns)
# :text              Switch back to text editor
# :tab               Open current file as CSV table

# ─── Clipboard ───────────────────────────────────────────────────────────────
# :clipboard       Open clipboard viewer (tool panel, shows ring entries)
# Ctrl+C           Copy selection (in any InputLine or editor visual mode)
# Ctrl+V / Ctrl+Y  Paste from clipboard ring
# "ayy             Yank to named register 'a'
# "ap              Paste from named register 'a'
#
# Tcl: system clipboard-get / system clipboard-set <text>
# MCP: clipboard_copy, clipboard_paste, clipboard_list
#
# set clipboard.max_entries 100   ;# default: 50
