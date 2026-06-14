//! Help topic: Tcl scripting reference.

pub(crate) fn help_tcl() -> String {
    let mut s = help_tcl_namespaces();
    s.push_str(help_tcl_examples());
    s
}

fn help_tcl_namespaces() -> String {
    "\
─── Tcl Scripting ──────────────────────────────
  Any M-x input not matching a built-in is
  evaluated as Tcl.

─── Namespaces ─────────────────────────────────
  editor   open, save, save-all, close, goto,
           insert, undo, redo, search,
           clear-highlight, current-file,
           current-line, current-col, modified?,
           filetype, get-selection,
           replace-selection, get-line,
           delete-line, replace-word, diff-revert

  view     focus, message, status, theme, zoom,
           toggle-tree, toggle-tools, layout

  build    run, test, test-file, test-at-cursor,
           next-error, prev-error

  lsp      hover, definition, references, rename,
           format, start, restart, stop, timeout,
           args, env
"
    .to_string()
}

fn help_tcl_examples() -> &'static str {
    "\
  git      stage, unstage, commit, blame,
           noblame, untrack, log

  todo     add, remove, complete,
           toggle-important, edit, swap,
           promote, demote, list

  split    vsplit, hsplit, close, focus, open,
           direction, linked

  keymap   bind, unbind
  hook     add, remove, list

  system   exec, env, set-env, root-dir, roots,
           add-root, remove-root, home-dir,
           platform, user, hostname, short-pwd,
           busy, clipboard-get, clipboard-set

─── Examples ─────────────────────────────────
  hook add file-save { build run }
  keymap bind ctrl+b { build run }
  editor goto 42

See also:
  → :help hooks       Hook events reference
  → :help commands    All M-x commands
"
}
