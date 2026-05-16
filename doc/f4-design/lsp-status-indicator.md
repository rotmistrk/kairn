# LSP Status Indicator

## Problem

When LSP servers are starting or indexing, user requests (goto definition, references, hover) time out silently after 10s with cryptic messages like "GotoShow: no response after 10s". The user has no visibility into whether the LSP is loading, indexing, or ready.

## Solution

A status bar item that shows per-language LSP state with live progress updates.

## Visual Design

Right-aligned in status bar, between existing items:

```
... │ rust ✓ go ⟳ │ Ln 42, Col 7 │ NOR │ ...
```

States and their display:

| State | Display | Meaning |
|-------|---------|---------|
| Starting | `rust …` | Server spawned, awaiting initialize response |
| Indexing | `rust ⟳` | Server sent $/progress begin |
| Indexing % | `rust 42%` | Server reports percentage |
| Ready | `rust ✓` | Initialized + no active progress |
| Error | `rust ✗` | Server died or failed to start |

Multiple languages shown space-separated: `rust ✓ go ⟳ ts ✓`

Empty label when no LSP servers are active (item invisible).

## Architecture

### State Model

```rust
// In src/lsp/status.rs (new file)
pub enum LspServerState {
    Starting,
    Indexing { percent: Option<u8>, message: Option<String> },
    Ready,
    Error,
}

pub struct LspStatusTracker {
    servers: HashMap<String, LspServerState>,
}
```

### Status Bar Item

```rust
// In src/status_items.rs or new src/status_items/lsp_status.rs
pub struct LspStatusItem {
    label: String,
}

impl ActiveItem for LspStatusItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        // Listen for CM_LSP_STATUS_UPDATE command
        // Update label from LspStatusSnapshot payload
    }
}

impl VisibleItem for LspStatusItem {
    fn label(&self) -> &str { &self.label }
    fn gravity(&self) -> Gravity { Gravity::Right }
}
```

### Progress Notification Handling

In `poll_lsp`, handle `$/progress` notifications:

```rust
LspMessage::Notification { method, params } => {
    match method.as_str() {
        "textDocument/publishDiagnostics" => { /* existing */ }
        "$/progress" => {
            // Parse WorkDoneProgress: begin/report/end
            // Update LspStatusTracker
            // Emit CM_LSP_STATUS_UPDATE
        }
        _ => {}
    }
}
```

### State Transitions

```
spawn → Starting
         │
         ├─ initialize response OK → Ready
         │                             │
         │                             ├─ $/progress begin → Indexing
         │                             │                       │
         │                             │                       ├─ report → Indexing(%)
         │                             │                       │
         │                             │                       └─ end → Ready
         │                             │
         │                             └─ server dies → Error
         │
         └─ initialize response ERR → Error
```

For servers that never send `$/progress`, state stays `Ready` after init.

### Timeout Behavior Change

While a server is in `Starting` or `Indexing` state:
- Extend timeout to 30s (or suppress timeout entirely)
- Show "LSP indexing, please wait…" instead of error on timeout
- Optionally queue the request for retry when state becomes Ready

When server is `Ready` and request times out:
- Use shorter timeout (10s)
- Show user-friendly message: "Go to definition timed out"

### Command Flow

```
CM_LSP_STATUS_UPDATE (payload: Vec<(String, LspServerState)>)
```

Emitted from `poll_lsp` whenever state changes. The status item rebuilds its label from the snapshot.

## Files to Create/Modify

| File | Change |
|------|--------|
| `src/lsp/progress.rs` | NEW — parse $/progress, LspStatusTracker |
| `src/lsp/handler.rs` | Handle $/progress notifications, emit status updates |
| `src/status_items.rs` | Add LspStatusItem |
| `src/status.rs` | Wire LspStatusItem into the bar |
| `src/commands.rs` | Add CM_LSP_STATUS_UPDATE |
| `src/lsp/registry.rs` | Expose state transitions (starting/ready/error) |

## Language Short Names

For compact display:

| Language ID | Display |
|-------------|---------|
| rust | rust |
| go | go |
| typescript | ts |
| javascript | js |
| python | py |
| c / cpp | c / c++ |
| java | java |

## Testing

- Unit test: LspStatusTracker state transitions
- Unit test: LspStatusItem label formatting
- Unit test: $/progress JSON parsing (begin/report/end)
- Integration: mock LSP sends progress → status bar shows indicator
