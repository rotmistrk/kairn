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

## Not Yet Implemented — To Prioritize

### Git Integration
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| File tree git status (colors + filter modes) | 55cef62 | Medium |
| Git commit log viewer | 0467516, 8cfcf14 | Medium |
| Git graph view (branch graph with colors) | 6395b78 | High |
| Blame mode (Tab cycles File→Diff→Log→Blame) | 9d62b33 | Medium |
| Diff mode | 530931c | Medium |

### Editor Enhancements
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Incremental search highlight (matches visible) | 2f9af9c | Low |
| CSV table view (Tab cycles to Table mode) | a3b6f37, bc47e95 | Medium |
| Lazy file loading (1000 lines initial) | bc47e95 | Medium |
| Auto-preview on tree cursor move | 17b9f48 | Low |
| Mtime-based cache invalidation | 1813b92 | Low |

### Terminal / Kiro Integration
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Region select + send to kiro/shell tab | 407fae8 | Medium |
| Template macros (@file @name @dir @line) | 4e5b57d, 8ee8e43 | Low |
| Kiro stderr capture (display in red) | 8822477 | Low |
| Kiro spawn error display | 293f06e | Low |
| Ctrl-] escape from terminal to main panel | f15fdff | Low |
| Terminal scrollback | ca12e67 | Medium |

### Session & Config
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Session save/restore (workspace state) | 2abe6bc, fbe4503 | Medium |
| Configurable keybindings (via init.tcl) | 116db8b | Low (mechanism exists) |
| Help: effective config display | f01386d | Low |

### Navigation & UX
| Feature | Legacy ref | Complexity |
|---------|-----------|-----------|
| Spatial navigation (Left/Right between panels) | 3804fb8 | Low |
| Two-chord keys (key sequences) | a5daf52 | Medium |
| Tree collapse-to-parent | a5daf52 | Low |
| Tree filter: hide empty dirs | 1b76fda | Low |
| Double-Esc to quit (fallback) | 6a33cd4 | Low |
| Manual refresh (F11) | 37e0a1a | Low |
| Status bar styling | 1b76fda | Low |

### New Ideas (not in legacy)
| Feature | Complexity |
|---------|-----------|
| LSP integration (diagnostics, go-to-def) | High |
| Find in files (grep) with results view | Medium |
| Compile command with error navigation | Medium |
| Multiple cursors | High |
| Snippet expansion | Medium |
| File watcher (inotify/kqueue) instead of polling | Medium |

---

## Priority (USER TO FILL)

_Reorder the above sections or mark P0/P1/P2/P3 per item._
