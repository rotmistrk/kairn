# kairn Feature Status & Development SOP

## Feature/Binding Status Table

### Key Bindings

| Key | Action | Status |
|-----|--------|--------|
| j/k | Tree: navigate | ✅ |
| Enter | Tree: open file (stay in tree) / expand dir | ✅ |
| Right | Tree: open file + focus editor / expand dir | ✅ |
| h/j/k/l | Editor: vim movement | ✅ |
| w/b/e/0/$/^ | Editor: word/line motions | ✅ |
| gg/G | Editor: top/bottom | ✅ |
| Arrow keys | Editor: movement (normal + insert) | ✅ |
| PgUp/PgDn | Editor: page scroll | ✅ |
| i/a/o/O/I/A | Editor: enter insert | ✅ |
| Esc | Editor: exit insert/visual | ✅ |
| x/dd/dw/d$/D | Editor: delete | ✅ |
| cc/cw/c$/C | Editor: change | ✅ |
| yy/yw/y$/Y | Editor: yank | ✅ |
| p/P | Editor: paste | ✅ |
| u / Ctrl-R | Editor: undo/redo | ✅ |
| . | Editor: dot repeat | ✅ |
| v/V | Editor: visual mode | ✅ |
| /pattern | Editor: search forward | ✅ |
| ?pattern | Editor: search backward | ✅ |
| n/N | Editor: next/prev match | ✅ |
| count prefix | Editor: repeat motions | ✅ |
| :w | Editor: save | ✅ |
| :q / :q! | Editor: close | ✅ |
| :wq | Editor: save + close | ✅ |
| :e file | Editor: open file (Tab completion) | ✅ |
| :N | Editor: go to line N | ✅ |
| :s/pat/rep/ | Editor: substitute | ✅ |
| :set wrap/nowrap | Editor: toggle wrap | ✅ |
| :set list/nolist | Editor: toggle list mode | ✅ |
| :set number/nonumber | Editor: toggle line numbers | ✅ |
| :!command | Editor: run shell command | ✅ |
| :.!command | Editor: filter line through command | ✅ |
| F1 | Show help view | ✅ |
| F2 | Focus tree (left slot) | ✅ |
| F3 | Focus main (center slot) | ✅ |
| F4 | Focus tools (right slot) | ✅ |
| F5 | Zoom toggle | ✅ |
| F6 | Show messages view | ✅ |
| Ctrl-Q | Quit | ✅ |
| M-x / ≈ / : | Command mode (with completion) | ✅ |
| Ctrl-Shift-Left | Focus previous slot | ✅ |
| Ctrl-Shift-Right | Focus next slot | ✅ |
| Ctrl-Shift-Up/Down | Dropdown tab picker | ✅ |

### Features

| Feature | Status | Notes |
|---------|--------|-------|
| File tree navigation | ✅ | Expand/collapse, open files |
| Vi editor (full) | ✅ | Motions, editing, visual, search, ex |
| Syntax highlighting | ✅ | syntect-based |
| Piece table buffer | ✅ | Undo/redo with grouping |
| Slotted desktop | ✅ | 4 slots, tabs, zoom, chrome |
| Dropdown tab picker | ✅ | LRU ordering, digit select |
| Composable status bar | ✅ | v-014 architecture |
| Mode indicator (NOR/INS/VIS) | ✅ | Right side of status bar |
| Position indicator (Ln/Col) | ✅ | Right side of status bar |
| Clock | ✅ | Right side, 60s interval |
| Git branch in status bar | ✅ | Reads .git/HEAD directly |
| Message view (F6) | ✅ | Bottom slot, scrollable |
| Welcome view | ✅ | Returns when center empty |
| File broker (no duplicates) | ✅ | Notified on close |
| Wide char rendering | ✅ | visual_positions in txv-core |
| Tab completion (:e, M-x) | ✅ | File paths + commands |
| AppSettings + EditorSettings | ✅ | 3-tier (global/defaults/instance) |
| Config file loading | ✅ | ~/.config/kairn/init.tcl (v-015 Phase 1) |
| Statusbar customization | ❌ | Planned (v-015 Phase 2) |
| Git status in tree | ❌ | |
| Real PTY shell tab | ❌ | Placeholder only |
| LSP integration | ❌ | |
| :split / :vsplit | ❌ | |
| Session persistence | ❌ | |

### Known Bugs

| Bug | Status | Notes |
|-----|--------|-------|
| (none currently) | | |

### Test Coverage

604 tests passing (as of 2026-05-10). Pre-commit hook enforces: fmt, clippy -D warnings, 240 code line limit, all tests pass.

## Development SOP

### Cycle

```
1. USER tests manually → reports issues + priority list
2. DEV writes failing tests for each issue (test-first)
3. DEV/AGENT implements fixes until tests pass
4. DEV verifies: cargo test --workspace (all pass)
5. DEV installs: cargo build --release && cp to kairn.f4
6. USER tests again → next cycle
```

### For each bug/feature:

```
1. Add to status table (this file)
2. Write failing test
3. Implement fix
4. Verify test passes
5. Update status table: ❌ → ✅
6. Commit
```

### 240 CODE Lines Per File Rule

**Reason:** Keep cognitive load low. Each file should do ONE thing well. When you
can read the entire file without scrolling much, you understand it fully. This
also keeps context small for AI agents working on the code.

**What counts:** Only lines with actual code. Blank lines, comment-only lines
(`//`, `//!`, `///`, `/* */`, `*`) do NOT count. The pre-commit hook enforces this.

**SOP when a file exceeds 240 code lines:**

```
DO NOT:
- Collapse code (shorter variable names, cramming logic onto one line)
- Remove comments or documentation
- Use macros just to reduce line count

DO:
- Split the file CONCEPTUALLY into two or more files
- Each new file should have a clear, nameable responsibility
- Use mod.rs to re-export if needed for API stability

Examples:
- editor/mod.rs too long → split into editor/movement.rs, editor/editing.rs
- status_items.rs too long → split into status_items/key_label.rs, status_items/clock.rs
- desktop/mod.rs too long → split into desktop/tabs.rs, desktop/layout.rs, desktop/chrome.rs
```

**The test:** If you can't name the new file with a clear noun or verb phrase
that describes its single responsibility, you're splitting wrong.
