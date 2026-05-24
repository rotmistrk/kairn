# kairn-prelude.tcl — Stub declarations for kairn bridge commands.
# Loaded by rusticle-lsp via --prelude to enable validation and completion.

# Editor operations
proc editor {subcmd args} {}
# Subcommands: open save save-all close goto insert undo redo search
#   clear-highlight current-file current-line current-col modified?
#   filetype get-selection replace-selection get-line delete-line
#   replace-word diff-revert

# View/UI operations
proc view {subcmd args} {}
# Subcommands: focus message status theme zoom layout
#   toggle-tree toggle-tools

# Build/test operations
proc build {subcmd args} {}
# Subcommands: run test test-file test-at-cursor next-error prev-error

# Project-wide search
proc grep {pattern args} {}

# LSP operations
proc lsp {subcmd args} {}
# Subcommands: start stop restart hover definition references rename
#   format timeout args

# Git operations
proc git {subcmd args} {}
# Subcommands: stage unstage untrack commit diff blame noblame log

# Todo operations
proc todo {subcmd args} {}
# Subcommands: add remove complete toggle-important edit list
#   promote demote swap

# Split/pane operations
proc split {subcmd args} {}
# Subcommands: vsplit hsplit close focus open direction linked only

# Key binding
proc keymap {subcmd args} {}
# Subcommands: bind unbind

# Event hooks
proc hook {subcmd args} {}
# Subcommands: add remove list

# System/environment
proc system {subcmd args} {}
# Subcommands: exec env set-env root-dir home-dir platform
#   clipboard-get clipboard-set
