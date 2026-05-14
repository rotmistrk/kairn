# Context-Based Status Bar Items

## Goal

Status bar items (key labels, indicators) should only appear and respond to events
when the focused view matches their context. E.g., git keys only show when the git
tree is focused; editor-specific keys only when an editor is focused.

## Design

### ViewContext type (txv-core)

Bitmask type — fast, no allocations:

```rust
pub type ViewContext = u64;

pub const CTX_EDITOR: ViewContext     = 1 << 0;
pub const CTX_TREE: ViewContext       = 1 << 1;
pub const CTX_TERMINAL: ViewContext   = 1 << 2;
pub const CTX_TODO: ViewContext       = 1 << 3;
pub const CTX_STRUCTURED: ViewContext = 1 << 4;
pub const CTX_GIT: ViewContext        = 1 << 5;
// extend as needed
```

### View trait addition (txv-core)

```rust
fn context(&self) -> ViewContext { 0 }
```

Default returns 0 (no context). Views override to declare what they are.

### StatusBarView trait (txv-core, new)

```rust
pub trait StatusBarView: View {
    fn set_active_context(&mut self, ctx: ViewContext);
}
```

### Program changes (txv-core)

- `status_bar` field becomes `Box<dyn StatusBarView>`
- Before draw: walk focused view chain, OR contexts → `set_active_context(ctx)`
- Before handle (preprocess): same

### StatusBar item filtering (txv-core)

- Add `context: ViewContext` field to item traits (default 0)
- In draw: skip items where `item.context != 0 && (item.context & active_ctx) == 0`
- In handle: skip items that don't match context
- Items with context=0 behave as today (always active/visible)

## Implementation Steps

1. txv-core: define `ViewContext` type and constants in a new `src/context.rs`
2. txv-core: add `fn context() -> ViewContext` to `View` trait (default 0)
3. txv-core: create `StatusBarView` trait extending `View`
4. txv-core: update `Program` to hold `Box<dyn StatusBarView>`, compute focus context
5. txv-core: update `StatusBar` — add `active_context` field, filter in draw/handle
6. txv-core: add `fn context() -> ViewContext` to `ActiveItem`/`VisibleItem` (default 0)
7. kairn: implement `context()` on EditorView, FileTreeView, TodoTree, TerminalView, StructuredView
8. kairn: annotate KeyLabelItems with context tokens in `build_status_bar`
9. kairn: update `Program::new()` call and return type of `build_status_bar`
10. Test and verify

## Notes

- Views can combine contexts: `CTX_EDITOR | CTX_GIT` for a diff view
- Groups can contribute context too if needed
- The simpler `txv_widgets::status_bar::StatusBar` is unused by kairn and can be ignored/removed
