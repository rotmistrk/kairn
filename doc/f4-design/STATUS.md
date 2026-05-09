# kairn Feature Status & Development SOP

## Feature/Binding Status Table

### Key Bindings

| Key | Action | Status | Notes |
|-----|--------|--------|-------|
| j/k | Tree: navigate | ✅ Done | |
| Enter | Tree: open file / expand dir | ✅ Done | No duplicate tabs |
| h/j/k/l | Editor: vim movement | ✅ Done | |
| Arrow keys | Editor: movement | ✅ Done | Both normal + insert |
| i/a/o/O/I/A | Editor: enter insert | ✅ Done | |
| Esc | Editor: exit insert | ✅ Done | |
| x/dd/dw/d$ | Editor: delete | ✅ Done | |
| u / Ctrl-R | Editor: undo/redo | ✅ Done | |
| :w | Editor: save | ✅ Done | |
| :q | Editor: close | ✅ Done | |
| F1 | Show help | ⚠️ Emits command, not implemented | Need help view |
| F2 | Focus tree (left slot) | ✅ Done | |
| F3 | Focus main (center slot) | ✅ Done | |
| F4 | Focus tools (right slot) | ✅ Done | |
| F5 | Zoom toggle | ✅ Done | |
| Ctrl-Q | Quit | ✅ Done | |
| M-x (≈) | Command mode | ❌ Not bound | Need to add StatusItem |
| Ctrl-Shift-Left | Previous tab | ❌ Not bound | Need to add StatusItem |
| Ctrl-Shift-Right | Next tab | ❌ Not bound | Need to add StatusItem |
| Ctrl-Shift-Up | Previous slot | ❌ Not bound | |
| Ctrl-Shift-Down | Next slot | ❌ Not bound | |
| Ctrl-Left | Shrink tree | ❌ Not bound | |
| Ctrl-Right | Grow tree | ❌ Not bound | |
| /pattern | Editor: search | ❌ Not implemented | |
| n/N | Editor: next/prev match | ❌ Not implemented | |
| :s/pat/rep/ | Editor: substitute | ❌ Not implemented | |
| gg/G | Editor: top/bottom | ⚠️ Pending 'g' leaks to screen | Bug |
| yy/p/P | Editor: yank/paste | ❌ Not verified | |
| v/V | Editor: visual mode | ❌ Not verified | |

### Features

| Feature | Status | Notes |
|---------|--------|-------|
| File tree | ✅ Done | Navigate, expand, open |
| Editor (vim) | ✅ Done | Basic editing works |
| Piece table buffer | ✅ Done | Undo/redo works |
| Slotted desktop | ✅ Done | 4 slots, tabs, zoom |
| Tab bar (chrome) | ✅ Done | Shows tab names |
| Status bar | ✅ Done | Shows key hints |
| Open file broker | ✅ Done | No duplicate tabs |
| Program (correct dispatch) | ✅ Done | Three-phase, re-dispatch |
| Shell terminal | ⚠️ Placeholder | Shows [Shell], no PTY |
| Command mode (M-x) | ❌ Binding missing | StatusBar has the code |
| Help view | ❌ Not implemented | F1 emits but no view |
| Completion | ❌ Not wired | Completer trait exists |
| Git status in tree | ❌ Not implemented | |
| Hidden files toggle | ❌ Not implemented | |
| :split / :vsplit | ❌ Not implemented | |
| :diff | ❌ Not implemented | |
| :blame | ❌ Not implemented | |
| LSP | ❌ Not wired | Module exists in git |
| Kiro integration | ❌ Not wired | Module exists in git |
| Autosave | ❌ Not implemented | |
| Session persistence | ❌ Not implemented | |

### Rendering Issues

| Issue | Status | Notes |
|-------|--------|-------|
| 'g' leaks to empty lines | 🐛 Bug | Vim pending key rendered |
| Tree cursor traces | ✅ Fixed | Full row fill |
| Wide char handling | ⚠️ Untested | |

## Development SOP (Standard Operating Procedure)

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
2. Write failing test in tests/scenarios/
3. Implement fix
4. Verify test passes
5. Update status table: ❌ → ✅
6. Commit with message: "fix: <description> (closes #N)"
```

### Agent instructions template:

```
Read doc/f4-design/steps/STATUS.md for the current feature table.
Read the failing tests in tests/scenarios/.

For each failing test:
1. Understand what it tests
2. Implement the fix
3. Verify: cargo test -p kairn --test <test_file>
4. Do NOT break other tests: cargo test -p kairn

After all fixes: cargo test --workspace must pass.
Commit. Do NOT ask questions.
```

### Rules:

- Every bug gets a failing test BEFORE the fix
- Every feature gets a test BEFORE implementation
- No fix without a test
- No manual-only verification
- Agent can do the implementation cycle autonomously
- User only does "feels right" testing
