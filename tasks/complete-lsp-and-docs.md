# Task: Complete LSP, Fix Quality Issues, Update Docs

**STATUS: DONE** — All parts verified implemented and tested (2026-05-12).

## Part 1: Wire Completion Popup

The CompletionPopup exists (`src/lsp/completion.rs`) with a `draw()` method
but is NEVER rendered. Wire it:

1. Add `completion_popup: CompletionPopup` field to EditorView
2. In EditorView::draw(), after drawing the editor, call `self.completion_popup.draw(surface)`
3. Handle CM_LSP_COMPLETION response: parse items, call `popup.show(items, cursor_x, cursor_y)`
4. Handle popup keys in EditorView::handle():
   - Up/Down: navigate items
   - Enter/Tab: accept selected item (insert text)
   - Esc: dismiss
   - Any other char: dismiss + process normally
5. Trigger: in insert mode, after typing (debounce via tick counter, ~5 ticks)
6. Test: mock completion response → verify popup visible

## Part 2: Ctrl+Key Handling Audit

The insert mode Ctrl fix was applied. Audit ALL key handling for similar bugs:

1. Normal mode: does Ctrl+key pass through correctly? (Ctrl-Q should quit, not be eaten)
2. Command mode (:): does Ctrl+key work? (Ctrl-C should cancel)
3. Search mode (/): same check
4. Visual mode: already has `if key.modifiers.ctrl { return Noop }` — good

For each mode, verify: Ctrl+key → Noop (passes to status bar preprocess).

## Part 3: didChange on Every Edit

Verify that `textDocument/didChange` is sent after each edit so the LSP
server has current content for diagnostics/completion:

1. Check if CM_CONTENT_CHANGED or similar triggers didChange
2. If not: add it — after every ContentChanged action, send didChange
3. Debounce: don't send on every keystroke, batch with tick (same as autosave pattern)

## Part 4: Server Shutdown on Quit

On CM_QUIT, send `shutdown` request + `exit` notification to all active LSP servers:

```rust
// In handler or main.rs before exit:
for client in state.lsp.active_clients() {
    client.send_request("shutdown", json!(null));
    client.send_notification("exit", json!(null));
}
```

## Part 5: Update Help Text

Update `src/help.rs` to include ALL current keybindings and commands:

### Key Bindings to Document

**Normal mode:**
- `gd` — Go to definition (LSP)
- `gr` — Find references (LSP)
- `K` — Hover info (LSP)

**Insert mode:**
- Ctrl+key — passes through (does NOT insert)

**Global:**
- F1-F6 — panel focus, help, messages
- F5 — zoom
- Ctrl-Q — quit
- Ctrl-Z — suspend to shell
- Ctrl-O — peek screen
- ≠/– — grow/shrink panel width
- ±/— — grow/shrink panel height
- M-0..9 — select tab by number
- Ctrl-Shift-Up/Down — dropdown tab picker
- Ctrl-Shift-Left/Right — cycle focus

**M-x Commands:**
- shell, kiro [--agent=name]
- close, tab-rename
- build, run, test, test-file, test-at-cursor
- next-error, prev-error
- lsp-rename, code-action
- grow, shrink, grow-v, shrink-v
- paste, messages, help, quit
- e/edit <file>, save

## Part 6: Update README.md

Brief overview of features, installation, key bindings reference.

## Constraints

- Pre-commit hook MUST pass
- 240 code line max
- No unwrap/expect/panic
- ALL existing tests must pass
- Add tests for completion popup and Ctrl+key handling

## MANDATORY: Scenario Tests for LSP

Every LSP feature MUST have a scenario test using a mock LSP server.
No feature is "done" without a passing test.

### Required Tests

1. **Server start**: open .rs file → verify rust-analyzer spawns (or mock)
2. **Server fail**: configure nonexistent server → verify error in messages
3. **gd (goto def)**: mock response with location → verify file opens at line
4. **gr (find refs)**: mock response with locations → verify results shown
5. **K (hover)**: mock response with text → verify displayed
6. **Ctrl-N (completion)**: mock response with items → verify popup
7. **didChange**: edit file → verify didChange sent to mock server
8. **Diagnostics**: mock publishDiagnostics → verify underline rendered
9. **Server crash**: kill mock server → verify graceful degradation, error in messages
10. **Disabled after fail**: spawn fails → verify no retry on next file open

### Mock Server Pattern

Create a simple mock LSP server (a script or binary) that:
- Reads JSON-RPC from stdin
- Responds with canned responses
- Can be configured per test

Or: use the existing `cat` trick from client.rs tests but with proper JSON-RPC framing.

### Exit Criteria

ALL 10 tests above MUST pass. If any LSP feature cannot be tested,
it is NOT done.
