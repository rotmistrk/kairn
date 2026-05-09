# Step 01: txv-core

**Reference**: `doc/f4-design/v-013-txv-architecture.md`

## What this is

The framework core. Pure logic. Zero external dependencies.
Defines the rules that everything else follows.

## Boundary

- **Creates**: `txv-core/` crate
- **Does NOT touch**: rusticle/, txv-render/, txv-widgets/, src/ (kairn)
- **Dependencies**: NONE. Pure Rust only.

## Deliverables

```
txv-core/src/
├── lib.rs          — re-exports, prelude, module-level docs with "how to create a View" example
├── geometry.rs     — Point, Rect (with contains, intersect, is_empty)
├── cell.rs         — Color, Attrs, Style, Cell (all Copy+Clone+PartialEq+Debug)
├── surface.rs      — Surface (owns cells, put, print, fill, hline, vline, sub, cell accessor, width, height, wide char handling)
├── event.rs        — KeyCode, KeyMod, KeyEvent, MouseButton, MouseAction, MouseEvent, CommandId, Event enum
├── view.rs         — ViewOptions, HandleResult, EventQueue, View trait, ViewState, delegate_view_state! macro
├── group.rs        — GroupState, delegate_group_state! macro, Group (three-phase dispatch)
├── window.rs       — WindowState, FrameStyle, delegate_window_state! macro
├── dialog.rs       — DialogState, delegate_dialog_state! macro
├── commands.rs     — well-known CommandIds (CM_QUIT, CM_CLOSE, CM_FOCUS_NEXT/PREV, CM_HELP, CM_MENU, CM_OK, CM_CANCEL)
└── run.rs          — Backend trait, run() function, exec_view(), MockBackend
```

## DRY requirements

### ViewState (every view embeds this)
```rust
pub struct ViewState {
    pub bounds: Rect,
    pub options: ViewOptions,
    pub dirty: bool,
    pub focused: bool,
    pub title: String,
}
```

### delegate_view_state! macro
Delegates: bounds, set_bounds, options, title, needs_redraw, mark_redrawn, select, unselect.
Every View impl uses this. No hand-written boilerplate.

### GroupState (every group embeds this)
```rust
pub struct GroupState {
    pub view: ViewState,
    pub children: Vec<Box<dyn View>>,
    pub focused: usize,
}
```

### delegate_group_state! macro
Delegates: all ViewState methods (via view field) + focus_next, focus_prev, insert, remove, child_count.
Also provides the three-phase handle() dispatch.

### WindowState, DialogState — same pattern.

## View trait (definitive)

```rust
pub trait View: Send {
    fn draw(&self, surface: &mut Surface);
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult;
    fn select(&mut self) {}
    fn unselect(&mut self) {}
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, rect: Rect);
    fn options(&self) -> ViewOptions;
    fn title(&self) -> &str { "" }
    fn needs_redraw(&self) -> bool { true }
    fn mark_redrawn(&mut self) {}
}
```

## Group three-phase dispatch

```
Phase 1: children with preprocess flag
Phase 2: modal child (if any) OR focused child
Phase 3: children with postprocess flag
```

Group calls select/unselect on focus change.
Group.needs_redraw() returns true if any child is dirty.

## EventQueue

Views call `queue.put_command(id, data)` to emit commands.
Run loop drains queue and re-dispatches after each handle cycle.

## run() and exec_view()

- `run()`: main event loop. Only redraws if needs_redraw(). Dispatches queued events.
- `exec_view()`: modal nested loop. Key/Mouse → modal only. Tick/Resize/Command → full tree.

## MockBackend

For testing without a terminal. Inject events, inspect surface.

## Verification

```bash
cargo test -p txv-core        # all tests pass
cargo clippy -p txv-core -- -D warnings
dupfinder txv-core/src        # no duplicates >5 lines
```

## Do NOT

- Do NOT add external dependencies
- Do NOT skip the macros (every State struct gets a delegate macro)
- Do NOT hand-write bounds/set_bounds/options/title/needs_redraw/select/unselect
- Do NOT use unwrap/expect outside tests
- Do NOT make HandleResult carry commands (views use queue.put_command)
