//! Help topic: hook events reference.

pub(crate) fn help_hooks() -> String {
    "\
─── Hook Events ──────────────────────────────
  hook add <event> ?-filter <pat>? <body>

  Events:
    file-save          After file is saved
    file-open          After file is opened
    file-close         After file tab is closed
    build-done         After build/test completes
    tab-switched       After active tab changes
    startup            On application start
    char-inserted      After character typed (filter: the char)
    char-deleted       After character deleted
    word-completed     After completion accepted (filter: word)
    idle               After idle timeout
    paste              After paste operation
    mode-changed       After editor mode change
    selection-changed  After selection changes
    lsp-start          When LSP server starts (filter: lang)

─── Filter Examples ──────────────────────────────
  hook add char-inserted -filter {(} { editor insert {)} }
  hook add word-completed -filter {TODO} {
    view message info hook {TODO detected}
  }
  hook add idle { lsp format }
  hook add lsp-start -filter {rust} {
    lsp env rust CARGO_TARGET_DIR /tmp/target
  }

─── Management ─────────────────────────────────
  hook add <event> <body>       Register hook
  hook remove <event>           Unregister all hooks for event
  hook list                     Show registered hooks

See also:
  → :help tcl         Tcl scripting reference
"
    .to_string()
}
