# TXV Hardening Proposal: Make Violations Impossible

## Problem Statement

TXV's design is correct but unenforceable. Agents (and humans) bypass it
because nothing stops them. The framework SUGGESTS patterns but ALLOWS
any code to:
- Set `dirty` directly
- Call `set_bounds` on any view from anywhere
- Call `select()`/`unselect()` manually
- Route events manually instead of using Group dispatch
- Access child views directly instead of through the framework

## Goal

Make every TXV violation a **compile error**. Not a lint. Not a code review
finding. A hard compiler rejection.

## Mechanism 1: Private Fields + Sealed Methods

### ViewState fields are PRIVATE

```rust
pub struct ViewState {
    bounds: Rect,        // PRIVATE — only set_bounds can change
    dirty: bool,         // PRIVATE — only mark_dirty() and mark_redrawn()
    focused: bool,       // PRIVATE — only select()/unselect() from parent
    options: ViewOptions,
    title: String,
}

impl ViewState {
    // The ONLY way to read bounds
    pub fn bounds(&self) -> Rect { self.bounds }

    // The ONLY way to mark dirty (called by framework after handle returns Consumed)
    pub(crate) fn mark_dirty(&mut self) { self.dirty = true }

    // Called by framework only
    pub(crate) fn set_focused(&mut self, f: bool) { self.focused = f }
}
```

No code outside txv-core can write `.dirty = true` or `.bounds = r`.
The delegate macros access these through the public API only.

### set_bounds is a framework-only call

```rust
pub trait View: Send {
    // Called by PARENT only — not by the view itself, not by siblings
    fn set_bounds(&mut self, rect: Rect);

    // Views that need to propagate to children override this
    // The framework calls it — user code NEVER calls set_bounds on another view
}
```

Enforcement: `set_bounds` takes a `&mut LayoutToken` parameter that only
the framework can create:

```rust
pub struct LayoutToken(());  // Cannot be constructed outside txv-core

pub trait View: Send {
    fn set_bounds(&mut self, rect: Rect, _token: &LayoutToken);
}
```

Now NO external code can call `view.set_bounds()` — they can't create a LayoutToken.
Only Group's internal layout code has access.

## Mechanism 2: Group Owns Children Exclusively

```rust
pub struct Group {
    children: Vec<Box<dyn View>>,  // PRIVATE — no direct access
    focused: usize,
}

impl Group {
    // The ONLY way to add a child
    pub fn insert(&mut self, view: Box<dyn View>) -> ChildId;

    // The ONLY way to remove
    pub fn remove(&mut self, id: ChildId) -> Box<dyn View>;

    // NO get_mut() — you cannot reach into a child and call methods on it
    // Communication is ONLY through events/commands
}
```

If you can't get `&mut child`, you can't call `child.set_bounds()` or
`child.select()` or `child.handle()`. The Group does ALL of that internally.

## Mechanism 3: Event-Only Communication

Views communicate ONLY through the EventQueue:
- Want to close a tab? Emit CM_TAB_CLOSE. The parent Group handles it.
- Want to focus a panel? Emit CM_FOCUS_LEFT. The parent handles it.
- Want to resize? Emit CM_PANEL_GROW. The parent handles it.

NO view ever reaches into another view. Period.

```rust
// WRONG (currently possible):
if let Some(v) = self.slots[i].active_view_mut() {
    v.unselect();  // Direct manipulation — VIOLATION
}

// RIGHT (with hardened TXV):
queue.put_command(CM_FOCUS_CHANGE, Some(Box::new(new_focus)));
// Group handles it internally — calls unselect/select as needed
```

## Mechanism 4: Layout Constraints (not manual rects)

Views don't set bounds on children. They declare CONSTRAINTS:

```rust
pub struct LayoutGroup {
    group: Group,
    constraints: LayoutConstraints,
}

pub struct LayoutConstraints {
    left_width: u16,
    right_width: u16,
    bottom_height: u16,
    zoomed: Option<usize>,
}
```

When constraints change, the Group's internal layout engine recomputes
bounds and calls set_bounds on children. The view code ONLY touches
constraints — never bounds directly.

```rust
// Resize command handler:
fn handle_resize(&mut self, delta: i16) {
    self.constraints.left_width += delta;
    self.invalidate_layout();  // triggers re-layout internally
}
```

## Mechanism 5: Compile-Time Macro Enforcement

The delegate macros generate ALL the boilerplate. If you don't use them,
you can't implement View (because ViewState fields are private):

```rust
// This WON'T COMPILE — can't access state.bounds directly
impl View for MyView {
    fn bounds(&self) -> Rect { self.state.bounds }  // ERROR: field is private
}

// This COMPILES — macro has access via pub(crate)
impl View for MyView {
    delegate_view_state!(state);  // generates correct accessors
    fn draw(&self, surface: &mut Surface) { ... }
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult { ... }
}
```

## Mechanism 6: No Raw View Trait Objects in User Code

User code works with `ChildId` handles, not `&mut dyn View`:

```rust
let editor_id = self.center_panel.insert(Box::new(editor_view));
// Later:
self.center_panel.set_active(editor_id);  // switches tab
self.center_panel.remove(editor_id);       // closes tab
// You CANNOT: self.center_panel.get_mut(editor_id).set_bounds(...)
```

## Summary: What Changes

| Before (broken) | After (hardened) |
|---|---|
| `self.state.dirty = true` | Impossible — field is private |
| `child.set_bounds(rect)` | Impossible — needs LayoutToken |
| `child.select()` | Impossible — Group handles internally |
| `child.handle(event, queue)` | Impossible — Group dispatches |
| `self.children[i]` direct access | Impossible — use ChildId |
| Manual layout in 5 places | Constraints + invalidate_layout() |

## Implementation Priority

1. Make ViewState fields private + pub(crate) accessors
2. Add LayoutToken to set_bounds signature
3. Remove get_mut from Group — use ChildId
4. Add LayoutConstraints pattern
5. Update all delegate macros
6. Rewrite desktop using the hardened API

## The Test

After hardening, try to write the OLD SlottedDesktop code.
It should NOT COMPILE. Every violation is a type error.
THAT is the definition of done.
