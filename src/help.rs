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
  F6              Messages window
  Ctrl-Shift-Left   Focus previous slot
  Ctrl-Shift-Right  Focus next slot

─── Tabs ─────────────────────────────────────────────
  Ctrl-Shift-Up/Down  Open tab dropdown picker
  Alt-0..9            Select tab by number
  Tab cycling         Use dropdown or Alt-N

─── Panel Resize ─────────────────────────────────────
  ≠ (Alt+=)         Grow focused slot width
  – (Alt+-)         Shrink focused slot width
  ± (Alt+Shift+=)   Grow focused slot height
  — (Alt+Shift+-)   Shrink focused slot height
  M-x grow/shrink   Same (command fallback)
  M-x grow-v/shrink-v  Vertical (command fallback)

─── Global ───────────────────────────────────────────
  F1              Show this help
  Ctrl-Q          Quit
  Ctrl-Z          Suspend to shell
  Ctrl-O          Peek screen (show terminal underneath)
  Ctrl-D          Diff current file vs HEAD
  M-x (Alt-x/≈)  Command mode prompt

─── Command Mode (M-x) ──────────────────────────────
  help            Show help
  quit            Quit
  edit <path>     Open file in editor (creates if new)
  e <path>        Short for edit
  save            Save current file
  close           Close current tab
  tab-rename <n>  Rename tool tab
  shell           New shell tab
  kiro [--agent=] New kiro session
  build           Run build command
  run             Run project
  test            Run tests
  test-file       Test current file
  test-at-cursor  Test at cursor
  next-error      Jump to next error
  prev-error      Jump to previous error
  lsp-rename <n>  Rename symbol (LSP)
  code-action     Show code actions (LSP)
  paste           Paste from system clipboard
  messages        Show messages window
  grow / shrink   Resize panel width
  grow-v / shrink-v  Resize panel height
  diff            Diff current file
  git-stage <p>   Stage file
  git-unstage <p> Unstage file
  git-commit <m>  Commit with message
  Tab             Complete command / file path

─── File Tree (left slot) ────────────────────────────
  j / Down        Move cursor down
  k / Up          Move cursor up
  Enter           Open file / expand directory
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
  c               Commit (opens prompt: type message, Enter)

─── Help / TextArea ──────────────────────────────────
  /               Search (type query, Enter to confirm)
  n / N           Next / previous match
  Esc             Cancel search input

─── Editor (center slot) — Normal Mode ───────────────
  h/j/k/l         Move left/down/up/right
  w / b / e       Word forward / backward / end
  0 / $ / ^       Line start / end / first non-blank
  gg / G          File start / end
  Ctrl-D/U        Half page down / up
  Ctrl-F/B        Full page down / up
  %               Match bracket
  f/F/t/T <ch>    Find char forward/back, till char
  ; / ,           Repeat / reverse last find

─── Editor — LSP (Normal Mode) ───────────────────────
  gd              Go to definition
  gr              Find references
  K               Hover info

  i / a           Insert before / after cursor
  I / A           Insert at line start / end
  o / O           Open line below / above
  x / X           Delete char forward / backward
  dd / dw / d$    Delete line / word / to end
  cc / cw / c$    Change line / word / to end
  yy              Yank line
  p / P           Paste after / before
  u / Ctrl-R      Undo / redo
  . (dot)         Repeat last edit
  r <ch>          Replace char under cursor
  J               Join lines
  ~               Toggle case
  >> / <<         Indent / unindent
  s / S           Substitute char / line
  v / V           Visual / visual-line mode

─── Editor — Visual Mode ─────────────────────────────
  h/j/k/l/w/b    Extend selection
  d / x           Delete selection
  y               Yank selection
  > / <           Indent / unindent selection
  Esc             Exit visual mode

─── Editor — Search ──────────────────────────────────
  /pattern        Search forward
  n / N           Next / previous match
  * / #           Search word under cursor fwd / back

─── Editor — Ex Commands (:) ─────────────────────────
  :w              Save
  :q              Close
  :wq / :x        Save and close
  :<N>            Go to line N
  :%s/pat/rep/g   Substitute (% = all lines)
  :d              Delete line(s)
  :y              Yank line(s)
  :diff            Diff vs HEAD (unified, 3 context lines)
  :diff -U5        Diff with 5 context lines
  :diff -w         Diff ignoring whitespace
  :diff --base <r> Diff vs branch/commit/remote

─── Editor — Insert Mode ─────────────────────────────
  Esc             Return to normal mode
  Arrow keys      Move cursor
  Backspace       Delete backward
  Delete          Delete forward
"
    .to_string()
}
