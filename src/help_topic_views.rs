//! Help topic: per-view key bindings (editor, tree, csv, struct, todo).

pub(crate) fn help_editor() -> String {
    let mut s = help_editor_normal();
    s.push_str(help_editor_extras());
    s
}

fn help_editor_normal() -> String {
    "\
─── Editor — Normal Mode ─────────────────────
  h/j/k/l         Move left/down/up/right
  w / b / e       Word forward / backward / end
  0 / $ / ^       Line start / end / first non-blank
  gg / G          File start / end
  f/F/t/T <ch>    Find char forward/back, till
  %               Match bracket

  i / a / I / A   Insert (before/after/start/end)
  o / O           Open line below / above
  dd / yy / p     Delete line / yank line / paste
  u / Ctrl-R      Undo / redo
  . (dot)         Repeat last edit
  v / V           Visual / visual-line mode
  >> / <<         Indent / unindent
  /pattern        Search forward
  n / N           Next / previous match

─── Editor — LSP ─────────────────────────────
  gd              Go to definition
  gr              Find references
  gR              Rename symbol
  K               Hover info
"
    .to_string()
}

fn help_editor_extras() -> &'static str {
    "\
─── Editor — Ex Commands (:) ─────────────────
  :w  :q  :wq     Save / close / save+close
  :%s/pat/rep/g   Substitute
  :diff            Diff vs HEAD
  :diff -y         Side-by-side diff
  :blame / :noblame

─── Editor — Diff Mode ───────────────────────
  j/k  n/N  g/G   Navigate
  R               Revert hunk
  Enter           Exit diff, jump to line

See also:
  → :help keys        All key bindings
"
}

pub(crate) fn help_tree() -> String {
    "\
─── File Tree ────────────────────────────────
  j / k           Navigate down / up
  Enter / →       Open file / expand directory
  h / ←           Collapse directory
  Ctrl-.          Toggle hidden (dot) files
  /               Filter (fuzzy search)

─── File Operations (Alt-f prefix) ─────────────
  n               New file (in cursor directory)
  N               New directory
  d               Delete file/directory
  r               Rename/move
  c               Copy
  m               Mark/unmark toggle
  u               Unmark all
  M               Move marked to cursor directory
  C               Copy marked to cursor directory

See also:
  → :help keys        All key bindings
"
    .to_string()
}

pub(crate) fn help_csv() -> String {
    "\
─── CSV/TSV Table View ────────────────────────
  h/j/k/l         Navigate cells
  g / G           First / last row
  0 / $           First / last column
  Enter           Edit cell
  s               Sort column (toggle asc/desc)
  f               Filter column
  F               Clear filter
  Ctrl-F          Clear all filters
  a               Add row
  d               Delete row (confirm)
  v               Visual select
  y / p           Yank / paste

See also:
  → :help keys        All key bindings
"
    .to_string()
}

pub(crate) fn help_struct() -> String {
    "\
─── Structured View (JSON/JSONC/JSONL) ────────
  j / k           Navigate nodes
  Space / l / →   Expand / collapse
  h / ←           Collapse / go to parent
  Tab             Cycle column (key ↔ value)
  Enter           Edit value
  n               New sibling
  b               New child
  d               Delete node
  t / T           Cycle type / toggle dict↔array
  J / K           Swap down / up
  H / L           Promote / demote
  u / Ctrl-R      Undo / redo
  s               Sort children
  f / F           Filter / clear filter

See also:
  → :help keys        All key bindings
"
    .to_string()
}

pub(crate) fn help_todo() -> String {
    "\
─── Todo Panel ─────────────────────────────────
  j / k           Navigate down / up
  Space           Toggle completed
  !               Toggle important
  e               Edit title (inline)
  n               New sibling task
  b               New child (subtask)
  d               Delete task
  J / K           Swap down / up
  H / L           Promote / demote
  N               Open note
  /               Filter

See also:
  → :help keys        All key bindings
"
    .to_string()
}
