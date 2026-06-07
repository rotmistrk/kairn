//! Help text: global, tree, terminal, git, todo, commands, and scripting.

use crate::help_global_extra;

/// Generate help text for global keys, panels, commands, and scripting.
pub fn help_global() -> String {
    let mut s = String::new();
    s.push_str(&help_navigation());
    s.push_str(&help_panels());
    s.push_str(&help_views());
    s.push_str(&help_commands());
    s.push_str(&help_global_extra::help_scripting());
    s.push_str(help_global_extra::help_hooks());
    s.push_str(help_global_extra::help_scope());
    s
}

fn help_navigation() -> String {
    let mut s = String::new();
    s.push_str(help_focus_tabs());
    s.push_str(help_resize_splits());
    s
}

fn help_focus_tabs() -> &'static str {
    "\
─── Slot Focus ─────────────────────────────────
  F2              Focus tree (left slot)
  F3              Focus main (center slot)
  F4              Focus terminal (right slot)
  F5              Zoom toggle (maximize focused slot)
  F6              Messages window
  Ctrl-Shift-Left   Focus previous panel (ring)
  Ctrl-Shift-Right  Focus next panel (ring)

─── Tabs ───────────────────────────────────
  Ctrl-Shift-Up/Down  Open tab dropdown picker
  Alt-0               Tab dropdown (list all tabs)
  Alt-1..9            Select tab by number
  Alt-;               Next tab
  Alt-'               Previous tab
  Alt-w               Close active tab

"
}

fn help_resize_splits() -> &'static str {
    "\
─── Panel Resize ────────────────────────────────
  Alt-Shift-Left    Move border left
  Alt-Shift-Right   Move border right
  Alt-Shift-Up      Move border up
  Alt-Shift-Down    Move border down
  ≠ (Alt+=)         Grow subpanel
  – (Alt+-)         Shrink subpanel

─── Splits & Layout ───────────────────────────────
  Ctrl-W …        Subpanel prefix:
    s               Split horizontal
    v               Split vertical
    c               Close this subpanel
    o               Close other (:only)
    w / Ctrl-W      Cycle focus
    m               Move tab to other
    + / -           Grow / shrink
    =               Equalize
  Alt-=             Grow subpanel
  Alt--             Shrink subpanel
  Alt-,             Toggle tree panel
  Alt-.             Toggle tools panel
  Alt-/             Zoom toggle
  Alt-\\             Cycle layout mode
"
}

fn help_panels() -> String {
    let mut s = String::new();
    s.push_str(help_global_tree_git());
    s.push_str(help_todo_terminal());
    s
}

fn help_global_tree_git() -> &'static str {
    "\
─── Global ────────────────────────────────
  F1              Show this help
  Ctrl-Q          Quit
  Ctrl-Z          Suspend to shell
  Ctrl-O          Peek screen (show terminal underneath)
  Ctrl-D          Diff current file vs HEAD
  Ctrl-L          Repaint screen
  M-x (Alt-x/≈)  Command mode prompt

─── File Tree (left slot, \"Files\" tab) ────────────────
  j / Down        Move cursor down
  k / Up          Move cursor up
  Enter / Right   Open file / expand directory
  h / Left        Collapse directory
  Ctrl-.          Toggle hidden (dot) files
  /               Filter (fuzzy search)
  Icons: set tree.icons true (requires Nerd Font)

─── Git Panel (left slot, \"Git\" tab) ─────────────────
  j / Down        Move cursor down
  k / Up          Move cursor up
  Enter           Open file (keep focus in tree)
  Right           Open file (focus editor)
  s               Stage file
  u               Unstage file
  x               Untrack file
  c               Commit (opens message prompt)

"
}

fn help_todo_terminal() -> &'static str {
    "\
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
"
}

fn help_views() -> String {
    let mut s = String::new();
    s.push_str(help_csv_view());
    s.push_str(help_structured_view());
    s
}

fn help_csv_view() -> &'static str {
    "\
─── CSV/TSV Table View ──────────────────────────
  j/k ↓/↑  Move row  | h/l ←/→  Move column
  g/G  First/last row | 0/$  First/last column
  Enter  Edit cell | s  Sort column (asc/desc toggle)
  f  Filter column | F  Clear filter | Ctrl-F  Clear all
  a  Add row | d  Delete (confirm) | v  Visual select
  Shift+↓/↑  Extend visual | y  Yank | p  Paste
  Esc  Cancel visual | :  Command mode

"
}

fn help_structured_view() -> &'static str {
    "\
─── Structured View (JSON/JSONC/JSONL) ──────────────────
  j/k ↓/↑  Navigate | g/G  First/last | Tab  Key↔Value
  Space/l/→  Expand | h/←  Collapse/parent | Enter  Edit
  n  New sibling | b  New child | d  Delete | c  Clone
  y  Yank JSON | p  Paste after | t  Cycle type | T  Dict↔Array
  J/K  Swap down/up | H/L  Promote/demote | !  Inline toggle
  s  Sort | S  Sort by path | f  Filter | F  Clear filter
  u  Undo | Ctrl-R  Redo | :  Command mode

"
}

fn help_commands() -> String {
    let mut s = String::new();
    s.push_str(help_commands_core());
    s.push_str(help_commands_extra());
    s
}

fn help_commands_core() -> &'static str {
    "\
─── Command Mode (M-x) ──────────────────────────────
  help            Show help
  quit            Quit
  edit <path>     Open file in editor (creates if new)
  e <path>        Short for edit
  save            Save current file
  close           Close current tab
  tab-rename <n>  Rename tool tab
  shell           New shell tab
  kiro [--agent=name]  New kiro session
  struct          Switch to structured view (JSON/CSV/TSV)
  text            Switch to plain text editor
  tab             Open current file as CSV/TSV table
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
  grep <pattern>  Search files for pattern
  grep -a <pat>   Search all workspace roots
  replace /p/r/   Find & replace across project (confirm each)
  add-root <path> Add workspace root directory
  remove-root <p> Remove workspace root

"
}

fn help_commands_extra() -> &'static str {
    "\
  diff            Diff current file
  blame           Show git blame annotations
  noblame         Hide git blame
  log             Show git log
  split [<file>]  Horizontal split
  vsplit [<file>] Vertical split
  zoom            Zoom toggle (maximize panel)
  layout          Cycle layout mode (auto/wide/tall)
  move-tab        Move tab to other subpanel
  cycle-subpanel  Cycle focus between subpanels
  grow-subpanel   Grow current subpanel
  shrink-subpanel Shrink current subpanel
  toggle-tree     Show/hide tree panel
  toggle-tools    Show/hide tools panel
  grow / shrink   Move panel border right/left
  grow-v / shrink-v  Move panel border down/up
  focus-left/right/up/down  Move focus between panels (ring)
  tab-next / tab-prev  Switch tabs in current panel
  theme <mode>    dark / light / auto / toggle
  git-stage <p>   Stage file
  git-unstage <p> Unstage file
  git-untrack <p> Untrack file
  git-commit <m>  Commit with message
  Tab             Complete command / file path

  Anything not recognized is evaluated as Tcl script.
"
}
