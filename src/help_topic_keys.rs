//! Help topic: key bindings reference.

pub(crate) fn help_keys() -> String {
    let mut s = help_keys_global();
    s.push_str(help_keys_panels());
    s
}

fn help_keys_global() -> String {
    "\
─── Global Keys ──────────────────────────────
  F1              Help
  F2 / F3 / F4   Focus: Tree / Editor / Terminal
  F5              Zoom (maximize focused panel)
  F6              Messages
  Ctrl-Q          Quit
  Ctrl-Z          Suspend to shell
  Ctrl-O          Peek (show terminal underneath)
  Ctrl-D          Diff current file vs HEAD
  Ctrl-L          Repaint screen
  M-x (Alt-x)    Command mode

─── Tabs ──────────────────────────────────
  Alt-0           Tab dropdown (list all tabs)
  Alt-1..9        Select tab by number
  Alt-;           Next tab
  Alt-'           Previous tab
  Alt-w           Close active tab
"
    .to_string()
}

fn help_keys_panels() -> &'static str {
    "\
─── Panel Navigation ────────────────────────
  Ctrl-Shift-←/→   Focus prev/next panel
  Alt-,             Toggle tree panel
  Alt-.             Toggle tools panel
  Alt-/             Zoom toggle
  Alt-\\             Cycle layout mode

─── Panel Resize ────────────────────────────
  Alt-Shift-←/→    Resize horizontally
  Alt-Shift-↑/↓    Resize vertically
  Alt-= / Alt--    Grow / shrink subpanel

─── Splits ──────────────────────────────────
  Ctrl-W s/v       Split horizontal/vertical
  Ctrl-W c/o       Close split / close others
  Ctrl-W w         Cycle focus
  Ctrl-W m         Move tab to other
  Ctrl-W +/-/=     Grow/shrink/equalize

See also:
  → :help editor      Vim editor keys
  → :help tree        File tree keys
  → :help csv         CSV view keys
  → :help struct      Structured view keys
  → :help todo        Todo panel keys
"
}
