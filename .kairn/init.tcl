# kairn project config — .kairn/init.tcl
# Tcl syntax. Overrides ~/.config/kairn/init.tcl for this project.

# ─── Build Commands ──────────────────────────────────────────────────────────
# Auto-detected from Cargo.toml / Makefile / package.json if not set.
# set build.command "cargo build"
# set run.command "cargo run"
# set test.command "cargo test"

# ─── Editor ──────────────────────────────────────────────────────────────────
# set editor.tabstop 4

# ─── Hooks ───────────────────────────────────────────────────────────────────
# hook add file-save { build run }
# keymap bind ctrl+b { build run }

# ─── LSP ─────────────────────────────────────────────────────────────────────
# lsp rust-analyzer {
#     command "rust-analyzer"
#     filetypes {rs}
# }
