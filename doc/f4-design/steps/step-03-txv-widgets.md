# Step 03: txv-widgets

**Reference**: `doc/f4-design/v-013-txv-architecture.md`
**Depends on**: Step 01 (txv-core) ONLY. Does NOT depend on txv-render.

## What this is

Concrete View implementations. Ready-to-use interactive components.
Each widget embeds ViewState and uses delegate_view_state! macro.

## Boundary

- **Creates**: `txv-widgets/` crate
- **Does NOT touch**: txv-core/, txv-render/, rusticle/, src/ (kairn)
- **Dependencies**: txv-core (path), ignore 0.4 (for FileTreeData/FileListData)
- **Does NOT depend on**: txv-render, crossterm, or any I/O crate

## Deliverables

```
txv-widgets/src/
├── lib.rs
├── scroll_view.rs      — ScrollView helper (not a View, embedded by others)
├── scrollbar.rs        — Scrollbar indicator
├── tree_view.rs        — TreeView<D: TreeData>
├── list_view.rs        — ListView<D: ListData>
├── input_line.rs       — InputLine (text input + history + completion)
├── tab_bar.rs          — TabBar (horizontal tabs)
├── status_bar.rs       — StatusBar (preprocess:true, StatusItems, key→command)
├── text_area.rs        — TextArea (read-only viewer, line numbers, search)
├── table.rs            — Table (columns + rows + selection)
├── menu.rs             — Menu (modal:true, popup)
├── dialog.rs           — Dialog (modal:true, buttons, input)
├── fuzzy_select.rs     — FuzzySelect (input + filtered list)
├── overlay.rs          — Overlay (positioned popup container)
├── progress_bar.rs     — ProgressBar (determinate/indeterminate)
├── split_pane.rs       — SplitPane (two areas, resizable divider)
├── file_tree.rs        — FileTreeData (TreeData impl for filesystem)
└── file_list.rs        — FileListData (ListData impl for file listing)
```

## DRY pattern (MANDATORY)

Every widget struct:
```rust
struct MyWidget {
    state: ViewState,  // MANDATORY
    // widget-specific fields
}

impl View for MyWidget {
    delegate_view_state!(state);  // MANDATORY — one line, no hand-written boilerplate

    fn draw(&self, surface: &mut Surface) { /* custom */ }
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult { /* custom */ }
}
```

## Special widgets

### StatusBar
- `options()` returns `ViewOptions { preprocess: true, focusable: false, .. }`
- Has `StatusItem { key: KeyEvent, command: CommandId, label: String }`
- `handle()`: if key matches a StatusItem, calls `queue.put_command(item.command, None)` and returns Consumed
- `draw()`: renders labels left-aligned, context right-aligned

### Menu
- `options()` returns `ViewOptions { modal: true, focusable: true, .. }`
- When active, captures all key/mouse events
- On selection: `queue.put_command(selected_item.command, None)`

### Dialog
- Uses DialogState (from txv-core)
- `options()` returns `ViewOptions { modal: true, focusable: true, .. }`
- On OK/Cancel: `queue.put_command(CM_OK/CM_CANCEL, None)`

## Event handling pattern

Widgets NEVER return commands. They call `queue.put_command()`:
```rust
fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
    if let Event::Key(key) = event {
        if key.code == KeyCode::Enter {
            queue.put_command(CM_OPEN_FILE, Some(Box::new(self.selected_path())));
            return HandleResult::Consumed;
        }
    }
    HandleResult::Ignored
}
```

## Verification

```bash
cargo test -p txv-widgets
cargo clippy -p txv-widgets -- -D warnings
dupfinder txv-widgets/src      # ZERO duplicates >5 lines allowed
```

## Do NOT

- Do NOT import txv-render or crossterm
- Do NOT hand-write bounds/set_bounds/options/title/needs_redraw/select/unselect
- Do NOT duplicate logic between widgets (extract to shared helper)
- Do NOT define new State structs without a corresponding delegate macro
- Do NOT use unwrap/expect outside tests
