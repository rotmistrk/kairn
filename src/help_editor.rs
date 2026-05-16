//! Help text: editor keybindings (normal, insert, visual, search, ex).

/// Generate help text for editor modes.
pub fn help_editor() -> String {
    "\
─── Editor — Normal Mode ─────────────────────────────
  h/j/k/l         Move left/down/up/right
  w / b / e       Word forward / backward / end
  0 / $ / ^       Line start / end / first non-blank
  gg / G          File start / end
  <N>G / <N>gg    Go to line N
  Ctrl-D/U        Half page down / up
  Ctrl-F/B        Full page down / up
  %               Match bracket
  f/F/t/T <ch>    Find char forward/back, till char
  ; / ,           Repeat / reverse last find

  i / a           Insert before / after cursor
  I / A           Insert at line start / end
  o / O           Open line below / above
  x / X           Delete char forward / backward
  s / S           Substitute char / line
  C               Change to end of line
  D               Delete to end of line
  dd / dw / d$    Delete line / word / to end
  db / d0 / d^    Delete word back / to start / non-blank
  cc / cw / c$    Change line / word / to end
  yy / yw / y$    Yank line / word / to end
  p / P           Paste after / before
  u / Ctrl-R      Undo / redo
  . (dot)         Repeat last edit
  r <ch>          Replace char under cursor
  J               Join lines
  ~               Toggle case
  >> / <<         Indent / unindent
  v / V           Visual / visual-line mode

─── Editor — LSP (Normal Mode) ───────────────────────
  gd              Go to definition
  gr              Find references
  gR              Rename symbol (prompts for name)
  K               Hover info

─── Editor — Visual Mode ─────────────────────────────
  h/j/k/l/w/b/e  Extend selection
  0 / $ / ^      Start / end / first non-blank
  G / gg          File end / start
  d / x           Delete selection
  c               Change selection
  y               Yank selection
  > / <           Indent / unindent selection
  :               Ex command on selection
  Esc             Exit visual mode

─── Editor — Search ──────────────────────────────────
  /pattern        Search forward
  n / N           Next / previous match
  * / #           Search word under cursor fwd / back

─── Editor — Ex Commands (:) ─────────────────────────
  :w              Save
  :q              Close (prompts if unsaved)
  :q!             Force close (discard changes)
  :wq / :x        Save and close
  :<N>            Go to line N
  :%s/pat/rep/g   Substitute (% = all lines)
  :d              Delete line(s)
  :y              Yank line(s)
  :set wrap/nowrap/number/nonumber/list/nolist
  :e <path>       Open file
  :diff            Diff vs HEAD (unified, 3 context lines)
  :diff -U5        Diff with 5 context lines
  :diff -w         Diff ignoring whitespace
  :diff --base <r> Diff vs branch/commit/remote
  :revert          Revert hunk under cursor (in diff mode)

─── Editor — Diff Mode ───────────────────────────────
  j / k           Move down / up
  n / N           Next / previous hunk
  g / G           Jump to start / end
  R               Revert hunk under cursor
  Enter           Exit diff, jump to line
  Esc             Exit diff mode
  /               Search
  :revert / :rev  Revert hunk (ex command)

─── Editor — Insert Mode ─────────────────────────────
  Esc             Return to normal mode
  Arrow keys      Move cursor
  Backspace       Delete backward
  Delete          Delete forward
  Tab             Insert tab character
  Ctrl-N / Ctrl-P Next / previous completion item

─── Editor — Completion Popup ────────────────────────
  Down / Up       Select next / previous item
  Enter / Tab     Accept completion
  Esc             Dismiss popup
"
    .to_string()
}
