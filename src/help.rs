//! Help text generator for kairn.

/// Generate the full help text listing all key bindings.
pub fn help_text() -> String {
    "\
╦╔═╔═╗╦╦═╗╔╗╔  Help
╠╩╗╠═╣║╠╦╝║║║
╩ ╩╩ ╩╩╩╚═╝╚╝

─── Slot Focus ───────────────────────────────────────
  F2              Focus tree (left slot)
  F3              Focus main (center slot)
  F4              Focus terminal (right slot)
  F5              Zoom toggle (maximize focused slot)
  Ctrl-Shift-Up   Focus previous slot
  Ctrl-Shift-Down Focus next slot

─── Tabs ─────────────────────────────────────────────
  Ctrl-Shift-Left   Previous tab in focused slot
  Ctrl-Shift-Right  Next tab in focused slot

─── Global ───────────────────────────────────────────
  F1              Show this help
  Ctrl-Q          Quit
  M-x (Alt-x/≈)  Command mode prompt

─── Command Mode (M-x) ──────────────────────────────
  help            Show help
  quit            Quit
  open <path>     Open file
  save            Save current file
  close           Close current tab
  shell           New shell tab

─── File Tree (left slot) ────────────────────────────
  j / Down        Move cursor down
  k / Up          Move cursor up
  Enter           Open file / expand directory
  h / Left        Collapse directory

─── Editor (center slot) — Normal Mode ───────────────
  h/j/k/l         Move left/down/up/right
  Arrow keys      Move left/down/up/right
  w / b           Word forward / backward
  0 / $           Line start / end
  gg / G          File start / end
  Ctrl-D/U        Half page down / up
  i / a           Insert before / after cursor
  I / A           Insert at line start / end
  o / O           Open line below / above
  x               Delete char forward
  dd              Delete line
  dw              Delete word
  yy              Yank line
  p               Paste
  u / Ctrl-R      Undo / redo
  :w              Save
  :q              Close

─── Editor — Insert Mode ─────────────────────────────
  Esc             Return to normal mode
  Arrow keys      Move cursor
  Backspace       Delete backward
  Delete          Delete forward
"
    .to_string()
}
