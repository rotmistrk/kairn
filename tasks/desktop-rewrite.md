# Task: Rewrite Desktop Using TXV Properly

## Problem

SlottedDesktop is a hand-rolled container that BYPASSES the TXV framework:
- Manages slots as raw Vec instead of Group children
- Manually propagates set_bounds (and forgets to in zoom, resize, etc.)
- Reimplements dispatch logic that Group already provides
- Breaks the composition model (delegate macros are useless)
- Every layout change requires manual propagation — guaranteed bugs

## Solution

Replace SlottedDesktop with a proper TXV composition:

```
Program
├── StatusBar (preprocess: true)
└── Desktop (Group with custom layout)
    ├── LeftPanel (Group — tabs: FileTree, GitChanges)
    ├── CenterPanel (Group — tabs: editors)
    └── RightPanel (Group — tabs: Shell, Kiro, Messages)
```

Each Panel is a **TabGroup** — a Group subclass that:
- Manages tabs (insert, remove, cycle, close)
- Draws tab chrome (title bar)
- Handles tab switching keys
- Delegates to active child View

The Desktop is a **LayoutGroup** — a Group subclass that:
- Computes layout (rects for each child panel)
- Calls child.set_bounds(rect) — THAT'S IT
- Handles zoom (give one child full bounds, others get 0x0)
- Handles focus cycling between panels
- Handles resize commands (adjust constraints, re-layout)

## Architecture

### TabGroup (new, in txv-core or txv-widgets)

```rust
pub struct TabGroup {
    group: GroupState,
    tabs: Vec<(String, Box<dyn View>)>,
    active: usize,
    lru: Vec<u64>,
    lru_counter: u64,
    chrome_style: Style,
}

impl View for TabGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle, needs_redraw });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.bounds = r;
        // Chrome takes row 0, content gets the rest
        let content = Rect::new(r.x, r.y + 1, r.w, r.h.saturating_sub(1));
        if let Some((_, view)) = self.tabs.get_mut(self.active) {
            view.set_bounds(content);
        }
        self.group.view.dirty = true;
    }

    fn draw(&self, surface: &mut Surface) {
        self.draw_chrome(surface);
        if let Some((_, view)) = self.tabs.get(self.active) {
            view.draw(surface);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Tab switching keys, then delegate to active view
    }
}
```

### LayoutGroup (the Desktop replacement)

```rust
pub struct LayoutGroup {
    group: GroupState,
    panels: [Box<dyn View>; 3],  // Left, Center, Right
    focused: usize,
    zoomed: Option<usize>,
    // Layout constraints
    left_width: u16,
    right_width: u16,
    right_height: u16,  // for tall mode vertical split
}

impl View for LayoutGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle, needs_redraw });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.bounds = r;
        let rects = self.compute_layout(r);
        for (i, panel) in self.panels.iter_mut().enumerate() {
            panel.set_bounds(rects[i]);
        }
        self.group.view.dirty = true;
    }

    // set_bounds is the ONLY place layout is computed.
    // Resize commands just change constraints and call self.set_bounds(self.bounds()).
    // Zoom just changes which panel gets full bounds.
    // NOTHING ELSE touches child bounds.
}
```

### Key Principle: set_bounds is the SINGLE source of truth

- Layout computed ONLY in set_bounds
- Resize → change constraint → call set_bounds
- Zoom → change flag → call set_bounds
- Terminal resize → Program calls set_bounds on root → propagates down
- NO manual propagation ANYWHERE else

### Dropdown

The dropdown tab picker is an **overlay** drawn on top after all panels.
It's NOT a child view — it's just a draw call in LayoutGroup::draw().

## Migration Plan

### Step 1: Create TabGroup in txv-widgets

- Implements View with delegate_group_state
- Manages tabs (insert, remove, active, LRU, can_close)
- Draws chrome (tab title bar)
- Handles: tab next/prev, dropdown, M-0..9, close
- Tests: unit tests for tab management

### Step 2: Create LayoutGroup in kairn (or txv-widgets)

- Three children: left, center, right TabGroups
- compute_layout: wide vs tall mode
- Zoom: one child gets full bounds
- Resize: adjust constraints, call set_bounds
- Focus cycling: F2/F3/F4, Ctrl-Shift-Left/Right
- Tests: layout computation, zoom, resize

### Step 3: Wire into Program

- Replace SlottedDesktop with LayoutGroup containing TabGroups
- build_desktop() creates the structure
- handler.rs uses the new API (insert_tab on the right TabGroup)

### Step 4: Delete SlottedDesktop

- Remove src/desktop/ entirely
- Remove all the manual propagation hacks

## Constraints

- Pre-commit hook MUST pass at every step
- 240 code line max per file
- No unwrap/expect/panic
- EVERY layout change goes through set_bounds — NO EXCEPTIONS
- Tests must cover: zoom, resize, tab switch, focus change

## Files to Create

```
txv-widgets/src/tab_group.rs      — TabGroup (tab container)
txv-widgets/src/tab_group/chrome.rs — tab bar drawing
src/layout_group.rs               — LayoutGroup (the desktop)
src/layout_group/layout.rs        — compute_layout
src/layout_group/dropdown.rs      — dropdown overlay
```

## Files to Delete (after migration)

```
src/desktop/mod.rs
src/desktop/chrome.rs
src/desktop/dispatch.rs
src/desktop/dropdown.rs
src/desktop/layout.rs
src/desktop/tabs.rs
```

## Non-Negotiable Rules

1. `set_bounds` is the ONLY method that touches child bounds
2. Every View uses delegate macros — no manual state management
3. Group dispatch handles event routing — no manual if/else chains
4. Resize/zoom = change constraint + call set_bounds. Period.
5. No "dirty" flag manipulation outside of set_bounds and handle
