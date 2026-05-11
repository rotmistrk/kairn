//! Help text: global, tree, terminal, git, todo, and M-x sections.

/// Generate help text for global keys, panels, and commands.
pub fn help_global() -> String {
    "\
─── Slot Focus ───────────────────────────────────────
  F2              Focus tree (left slot)
  F3              Focus main (center slot)
  F4              Focus terminal (right slot)
  F5              Zoom toggle (maximize focused slot)
  F6              Messages window
  Ctrl-Shift-Left   Focus previous slot
  Ctrl-Shift-Right  Focus next slot

─── Tabs ─────────────────────────────────────────────
  Ctrl-Shift-Up/Down  Open tab dropdown picker
  Alt-0..9            Select tab by number

─── Panel Resize ─────────────────────────────────────
  ≠ (Alt+=)         Grow focused slot width
  – (Alt+-)         Shrink focused slot width
  ± (Alt+Shift+=)   Grow focused slot height
  — (Alt+Shift+-)   Shrink focused slot height

─── Global ───────────────────────────────────────────
  F1              Show this help
  Ctrl-Q          Quit
  Ctrl-Z          Suspend to shell
  Ctrl-O          Peek screen (show terminal underneath)
  Ctrl-D          Diff current file vs HEAD
  M-x (Alt-x/≈)  Command mode prompt

─── File Tree (left slot, \"Files\" tab) ────────────────
  j / Down        Move cursor down
  k / Up          Move cursor up
  Enter / Right   Open file / expand directory
  h / Left        Collapse directory
  Ctrl-.          Toggle hidden (dot) files

─── Git Panel (left slot, \"Git\" tab) ─────────────────
  j / Down        Move cursor down
  k / Up          Move cursor up
  Enter           Open file (keep focus in tree)
  Right           Open file (focus editor)
  s               Stage file
  u               Unstage file
  x               Untrack file
  c               Commit (opens message prompt)

─── Todo Panel (left slot, \"Todo\" tab) ────────────────
  j / Down        Move cursor down
  k / Up          Move cursor up
  Enter / Right   Expand / collapse
  Space           Toggle completed
  !               Toggle important
  e               Edit title (inline)
  n               New sibling task
  b               New child (subtask)
  d               Delete task
  J / K           Swap down / up
  H / L           Promote / demote (change nesting)

─── Terminal (right slot) ────────────────────────────
  PgUp / PgDn    Scroll back / forward
  (all other keys pass through to the shell/kiro)

─── Command Mode (M-x) ──────────────────────────────
  help            Show help
  quit            Quit
  edit <path>     Open file in editor (creates if new)
  e <path>        Short for edit
  save            Save current file
  close           Close current tab
  tab-rename <n>  Rename tool tab
  shell           New shell tab
  kiro [agent]    New kiro session (default: kairn agent)
  build           Run build command
  run             Run project
  test            Run tests
  test-file       Test current file
  test-at-cursor  Test at cursor
  next-error      Jump to next diagnostic/error
  prev-error      Jump to previous diagnostic/error
  lsp-rename <n>  Rename symbol (LSP)
  lsp-status      Show LSP server status
  code-action     Show code actions (LSP)
  paste           Paste from system clipboard
  messages        Show messages window
  grow / shrink   Resize panel width
  grow-v / shrink-v  Resize panel height
  diff            Diff current file
  git-stage <p>   Stage file
  git-unstage <p> Unstage file
  git-untrack <p> Untrack file
  git-commit <m>  Commit with message
  Tab             Complete command / file path
"
    .to_string()
}
