//! Help text: global, tree, terminal, git, todo, commands, and scripting.

/// Generate help text for global keys, panels, commands, and scripting.
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
  Ctrl-L          Repaint screen
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
  diff            Diff current file
  theme <mode>    dark / light / auto / toggle
  git-stage <p>   Stage file
  git-unstage <p> Unstage file
  git-untrack <p> Untrack file
  git-commit <m>  Commit with message
  Tab             Complete command / file path

  Anything not recognized is evaluated as Tcl script.

─── Tcl Scripting ────────────────────────────────────
  Any M-x input that isn't a built-in command is
  evaluated as Tcl. Available namespaces:

  editor open <path> ?-line N?   Open file
  editor save / save-all / close / undo / redo
  editor goto <line> ?<col>?     Jump to position
  editor insert <text>           Insert at cursor
  editor diff-revert             Revert diff hunk under cursor
  editor current-file / current-line / current-col
  editor modified? / filetype

  view focus <slot>              left/center/right
  view message <level> <origin> <text>
  view status <text>             Flash in status bar

  build run ?<cmd>? / build test ?<cmd>?
  lsp hover / definition / references
  lsp rename <name> / lsp format

  git stage <file> / unstage <file> / commit <msg>
  todo add <text> / remove <path> / complete <path>

  keymap bind <key> <command>    Bind key to command
  keymap unbind <key>            Remove binding

  hook add <event> <body>        Register hook
  hook remove <event>            Unregister hooks
  Events: file-save, file-open, file-close,
          build-done, tab-switched, startup

  system exec <cmd>              Run shell command
  system env <var>               Get env variable
  system root-dir / home-dir / platform
  system clipboard-get / clipboard-set <text>

─── Editor Selection & Line Commands ─────────────────
  editor get-selection        Return selected text
  editor replace-selection <text>  Replace selection
  editor get-line ?<n>?       Get line content (default: cursor)
  editor delete-line ?<n>?    Delete line (default: cursor)
  editor replace-word <text>  Replace word under cursor

  These work from Tcl scripts and keybindings:
    keymap bind ctrl+q {
      set sel [editor get-selection]
      editor replace-selection [concat {'} $sel {'}]
    }

─── Filtered Hooks ───────────────────────────────────
  hook add <event> ?-filter <pat>? <body>

  Events with filter support:
    char-inserted   Filter: the character (e.g. open paren)
    word-completed  Filter: the completed word
    idle            No filter (fires after idle timeout)

  Examples:
    hook add char-inserted -filter {(} {
      editor insert {)}
    }
    hook add word-completed -filter {TODO} {
      view message info hook {TODO detected}
    }
    hook add idle { lsp format }

─── Command Scope ────────────────────────────────────
  Commands can be entered two ways:
    :command    From editor or structured view (vim-style)
    M-x command From anywhere (status bar prompt)

  Editor : handles local commands first, then forwards
  unknown commands to M-x dispatch. Shell/kiro tabs
  pass all keys to the PTY — use M-x from those.

  Scope table:
    Editor-only:  :123 (goto line), :s/pat/rep/ (substitute),
                  :d (delete lines), :y (yank lines),
                  :set (local option), :! (shell filter)
    View-local:   save, close, diff, nodiff, struct, text
    App-global:   shell, kiro, build, test, grep, edit,
                  lsp-rename, code-action, paste, messages,
                  grow, shrink, git-*
"
    .to_string()
}
