# Feature Backlog — Post-Merge

## Already Implemented (master)

| Feature | Legacy ref |
|---------|-----------|
| Full vi editor (motions, editing, visual, search, ex) | ✅ |
| File tree navigation | ✅ |
| Syntax highlighting | ✅ |
| Real PTY shell + VTE terminal emulation | 4ae2d63 |
| Systematic tab names (Shell:N, Kiro:N) | 4f4e09f |
| Tab rename (M-x rename) | a6e2d37 |
| OSC 52 clipboard + bracketed paste | c420a1e |
| Panel resize (≠–) | 692446e, aa0a069 |
| Suspend to shell (Ctrl-Z) + nesting guard | e52b03b, 2693e3f |
| Peek screen (Ctrl-O) | 921cc5c |
| Panic handler | 9da20f4 |
| Welcome screen | 3ff0962 |
| CLI argument parsing (clap) | 01c1c7c |
| Autosave | new (not in legacy) |
| Tab close protocol (can_close, LRU) | new |
| Non-blocking PTY writes | new |
| Tree auto-refresh | 37e0a1a |
| Config loading (init.tcl) | 116db8b |
| M-0..9 tab select | new |
| File tree git status (colors + filter modes) | 55cef62 |
| Blame mode | 9d62b33 |
| Diff mode | 530931c |
| Git commit log viewer | 0467516, 8cfcf14 |
| CSV table view | a3b6f37, bc47e95 |
| Session save/restore | 2abe6bc, fbe4503 |
| Configurable keybindings (via init.tcl) | 116db8b |
| LSP integration (diagnostics, go-to-def, references, rename) | new |
| Find in files (grep) with results view | new |
| Compile command with error navigation | new |
| Terminal scrollback | ca12e67 |
| Spatial navigation (Ctrl-Shift-Left/Right) | 3804fb8 |
| Split/vsplit editor panes | new |

## Not Yet Implemented — To Prioritize

### Git Integration
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Git graph view (branch graph with colors) | 6395b78 | High |

### Editor Enhancements
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Lazy file loading (1000 lines initial) | bc47e95 | Medium |
| Auto-preview on tree cursor move | 17b9f48 | Low |
| Mtime-based cache invalidation | 1813b92 | Low |
| Multiple cursors | — | High |
| Snippet expansion | — | Medium |

### Terminal / Kiro Integration
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Region select + send to kiro/shell tab | 407fae8 | Medium |
| Template macros (@file @name @dir @line) | 4e5b57d, 8ee8e43 | Low |

### Navigation & UX
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Two-chord keys (key sequences) | a5daf52 | Medium |
| Tree filter: hide empty dirs | 1b76fda | Low |
| Status bar styling / customization | 1b76fda | Low |
| File watcher (inotify/kqueue) instead of polling | — | Medium |

---

## Priority (USER TO FILL)

_Reorder the above sections or mark P0/P1/P2/P3 per item._
