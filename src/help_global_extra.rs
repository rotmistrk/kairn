//! Help text: scripting, hooks, and command scope.

pub fn help_scripting() -> String {
    let mut s = String::new();
    s.push_str(help_scripting_core());
    s.push_str(help_scripting_extra());
    s
}

fn help_scripting_core() -> &'static str {
    "\
─── Tcl Scripting ────────────────────────────────
  Any M-x input that isn't a built-in command is
  evaluated as Tcl. Available namespaces:

  editor open <path> ?-line N?   Open file
  editor save / save-all / close / undo / redo
  editor goto <line> ?<col>?     Jump to position
  editor insert <text>           Insert at cursor
  editor search <pattern>        Search in buffer
  editor clear-highlight         Clear search highlight
  editor diff-revert             Revert diff hunk under cursor
  editor current-file / current-line / current-col
  editor modified? / filetype

  view focus <slot>              left/center/right
  view message <level> <origin> <text>
  view status <text>             Flash in status bar

  build run ?<cmd>? / build test ?<cmd>?
  lsp hover / definition / references
  lsp rename <name> / lsp format
  lsp start ?<pattern>?          Start LSP server
  lsp restart ?<pattern>?        Restart LSP server
  lsp stop ?<pattern>?           Stop LSP server

"
}

fn help_scripting_extra() -> &'static str {
    "\
  git stage <file> / unstage <file> / commit <msg>
  git blame / noblame
  todo add <text> / remove <path> / complete <path>

  split vsplit ?<file>?          Vertical split
  split hsplit ?<file>?          Horizontal split
  split close / split only       Close split
  split focus                    Cycle split focus
  split open <path>              Open file in other pane
  split linked ?<bool>?          Toggle linked scroll

  keymap bind <key> <command>    Bind key to command
  keymap unbind <key>            Remove binding

  hook add <event> <body>        Register hook
  hook remove <event>            Unregister hooks
  Events: file-save, file-open, file-close,
          build-done, tab-switched, startup,
          char-inserted, char-deleted,
          word-completed, idle, paste,
          mode-changed, selection-changed

  system exec <cmd>              Run shell command
  system env <var>               Get env variable
  system root-dir / home-dir / platform
  system clipboard-get / clipboard-set <text>
"
}

pub fn help_hooks() -> &'static str {
    "\
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

─── Filtered Hooks ─────────────────────────────
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
"
}

pub fn help_scope() -> &'static str {
    "\
─── Command Scope ──────────────────────────────
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
}
