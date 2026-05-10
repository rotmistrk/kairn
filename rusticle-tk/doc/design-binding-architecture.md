# rusticle-tk Bridge Redesign — Design Options

## Problem

The current `tk_bridge` is a central module that knows every widget type.
It downcasts `dyn View` → concrete type to call widget-specific methods.
This violates high cohesion: one module knows everything.

## Goal

Each widget type should be self-contained: owns its typed widget, registers
its own Tcl commands, parses its own parameters. No central dispatcher.

---

## Option A: Binding-as-View

Each widget gets a binding struct that wraps the widget and implements View:

```rust
struct TextBinding {
    state: ViewState,
    widget: TextArea,
}

impl View for TextBinding {
    delegate_view_state!(state);
    fn draw(&self, s: &mut Surface) { self.widget.draw(s) }
    fn handle(&mut self, e: &Event, q: &mut EventQueue) -> HandleResult {
        self.widget.handle(e, q)
    }
}
```

The binding lives in GroupState as `Box<dyn View>`. Tcl commands access it
via `desktop.get_mut(id).as_any_mut().downcast_mut::<TextBinding>()`.

**Pro**: Binding is self-contained. Only TextBinding knows about TextArea.
**Con**: Still uses downcast (but to self, not to inner widget).

---

## Option B: Binding owns storage independently

Each binding manages its own `Arc<Mutex<WidgetState>>`. The Tcl command
closures capture this directly — no downcast needed. A thin View wrapper
in the desktop borrows from the Arc for draw/handle.

```rust
struct TextState { widget: TextArea }
struct TextViewProxy { state: Arc<Mutex<TextState>> }

impl View for TextViewProxy { /* lock and delegate */ }
```

Tcl closures capture `Arc<Mutex<TextState>>` directly.

**Pro**: Zero downcast. Tcl closures have direct typed access.
**Con**: Mutex lock on every draw/handle. Two objects per widget.

---

## Option C: Command-based interface (no typed access)

Widgets only communicate via commands. Script commands emit events like
`CM_TEXT_SET_CONTENT` with data payload. The widget handles them in its
`handle()` method. Getting data back uses a response channel or shared slot.

**Pro**: Pure TXV architecture. No downcast. No typed access needed.
**Con**: Complex for simple get/set. Needs response mechanism for queries.

---

## Option D: Trait-based widget protocol

Define a `ScriptWidget` trait with `get_property`/`set_property`:

```rust
trait ScriptWidget: View {
    fn set_property(&mut self, key: &str, value: &str) -> Result<(), String>;
    fn get_property(&self, key: &str) -> Result<String, String>;
}
```

Each binding implements this. The bridge calls it generically.

**Pro**: No downcast. Central bridge stays generic.
**Con**: Stringly-typed. Loses compile-time safety on property names.

---

## Current Decision

Leave as-is (Option A without the binding wrapper — direct downcast to
concrete widget types). Revisit when rusticle-tk becomes the focus.

## Recommendation

Option A is the likely winner: each binding module is self-contained,
implements View, and the only downcast is to the binding itself (not to
the inner widget). The central `tk_bridge/mod.rs` just calls
`TextBinding::register()`, `ListBinding::register()`, etc.
