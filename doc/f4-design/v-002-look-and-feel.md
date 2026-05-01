# v-002 — Look and Feel

## Screen model

The screen is a grid of cells. Each cell has: a character (Unicode), a
foreground color, a background color, and attributes (bold, italic, underline,
reverse). The rendering engine owns two grids: current and previous. On each
frame, it diffs them and emits only the changed cells as terminal escape
sequences.

The grid is managed by the `txv` crate. kairn writes to the grid through a
`Surface` API — no widget abstraction layer. Panels render themselves by
writing cells directly.

Terminal size changes trigger a full grid reallocation and redraw. There is
no incremental resize — the entire screen is repainted. This eliminates the
class of garbage-on-resize bugs seen with ratatui.

## Coordinate system

All positions are `(row, col)` with `(0, 0)` at top-left. The status bar
occupies the last row. The tab bar (if visible) occupies the first row.
Panel areas are computed from the remaining space by the layout engine.

## Border rendering

Two modes, toggled by a key (e.g., `Ctrl-X B`):

### Pretty mode (default)

```
┌─ Files ──────┬─ src/main.rs [+] ──────────────┬─ Outline ──────┐
│ 📁 src/      │  1 │ fn main() {                │ fn main()      │
│   🦀 main.rs │  2 │     let app = App::new();  │ fn run_loop()  │
│   🦀 lib.rs  │  3 │     run_loop(&mut app);    │ fn render()    │
│ 📄 Cargo.toml│  4 │ }                          │                │
│              │  5 │                            │                │
├──────────────┴─────────────────────────────────┴────────────────┤
│ terminal  kiro  errors  search                                  │
│ $ cargo build                                                   │
│    Compiling kairn v0.1.0                                       │
├─────────────────────────────────────────────────────────────────┤
│ vi:NORMAL  src/main.rs  4:1  master  UTF-8           Ctrl-X B:cp│
└─────────────────────────────────────────────────────────────────┘
```

Uses box-drawing characters. Looks sharp but copies as garbage.

### Copy-friendly mode

```
  Files           src/main.rs [+]                   Outline
  📁 src/         1 │ fn main() {                  fn main()
    🦀 main.rs    2 │     let app = App::new();    fn run_loop()
    🦀 lib.rs     3 │     run_loop(&mut app);      fn render()
  📄 Cargo.toml   4 │ }
                   5 │
  terminal  kiro  errors  search
  $ cargo build
     Compiling kairn v0.1.0
 vi:NORMAL  src/main.rs  4:1  master  UTF-8           Ctrl-X B:cp
```

- Vertical borders: single column of differently-colored spaces
- Horizontal borders: full-width row of differently-colored spaces
- Panel titles rendered as colored text on the border row
- Active panel: brighter border color (e.g., bright blue bg)
- Inactive panel: dim border color (e.g., dark gray bg)

The line-number gutter separator `│` is always present in both modes —
it's part of the editor content, not a border.

## Color scheme

Base palette (gruvbox-inspired, configurable via Tcl themes):

| Element | Foreground | Background |
|---------|-----------|------------|
| Editor text | #ebdbb2 (light cream) | #282828 (dark bg) |
| Line numbers | #665c54 (gray) | #282828 |
| Current line | — | #3c3836 (slightly lighter) |
| Active border | #83a598 (blue-green) | — |
| Inactive border | #504945 (dark gray) | — |
| Status bar | #ebdbb2 | #504945 |
| Tab bar (active) | #282828 | #83a598 |
| Tab bar (inactive) | #a89984 | #3c3836 |
| Selection | — | #458588 (blue) |
| Search match | #282828 | #fabd2f (yellow) |
| Current search match | #282828 | #fe8019 (orange) |
| Error | #fb4934 (red) | — |
| Warning | #fabd2f (yellow) | — |
| Modified indicator | #fabd2f | — |
| Git added | #b8bb26 (green) | — |
| Git removed | #fb4934 (red) | — |
| Git modified | #83a598 (blue) | — |

Themes are Tcl scripts that set color variables:

```tcl
# gruvbox.tcl
theme set editor-fg "#ebdbb2"
theme set editor-bg "#282828"
theme set border-active "#83a598"
# ...
```

## Panel layout

### Layout modes

Three modes, cycled with `Ctrl-L`:

```
Wide (default for terminals > 160 cols):
┌──────┬──────────────────────────────┬───────────────┐
│ Tree │          Editor              │   Control     │
│      │                              │               │
├──────┴──────────────────────────────┴───────────────┤
│ Bottom panel                                        │
└─────────────────────────────────────────────────────┘

Tall-Right (default for 100-160 cols):
┌──────┬──────────────────────────────────────────────┐
│ Tree │          Editor                              │
│      │                                              │
│      ├──────────────────────────────────────────────┤
│      │ Bottom panel                                 │
└──────┴──────────────────────────────────────────────┘

Tall-Bottom (default for < 100 cols):
┌──────┬──────────────────────────────────────────────┐
│ Tree │          Editor                              │
├──────┴──────────────────────────────────────────────┤
│ Bottom panel                                        │
└─────────────────────────────────────────────────────┘
```

Control panel is hidden in Tall-Right and Tall-Bottom (not enough space).
It can be toggled on, which pushes the editor narrower.

### Panel sizing

- Tree: default 20 cols, resizable with F7/F8 (Shift: ×5), min 10, max 40
- Control: default 25 cols, resizable, min 15, max 40
- Bottom: default 30% of height, resizable with F9/F10, min 3 rows, max 70%
- Editor: takes remaining space

### Panel visibility toggles

| Key | Action |
|-----|--------|
| `Ctrl-B` | Toggle tree panel |
| `Ctrl-X P` | Toggle control panel |
| `Ctrl-X \` | Toggle bottom panel |
| `Ctrl-L` | Cycle layout mode |
| `Ctrl-X W` | Toggle windows/tabs mode |

### Windows vs tabs

In **tabs mode** (default): one triptych visible at a time, tab bar at top
shows all open files. `Ctrl-Shift-↑/↓` or `Alt-Left/Right` switches tabs.

In **windows mode**: screen splits to show multiple triptychs. Each window
has its own tree+editor+control. The bottom panel remains shared.

## Status bar

Single row at the bottom of the screen. Left-aligned info, right-aligned
controls.

```
 vi:INSERT [+]  src/editor/mod.rs  142:17  master  UTF-8    F1:help  Ctrl-Q:quit
```

### Left section

| Field | When shown | Example |
|-------|-----------|---------|
| Keymap mode | Always | `vi:NORMAL`, `vi:INSERT`, `emacs`, `classic` |
| Modified | When buffer dirty | `[+]` |
| File path | When file open | `src/editor/mod.rs` |
| Line:Col | When file open | `142:17` |
| Git branch | When in git repo | `master` |
| Encoding | Always | `UTF-8` |
| Diagnostics | When LSP active | `⚠2 ✕1` |

### Right section

| Field | When shown | Example |
|-------|-----------|---------|
| Pending chord | During two-key sequence | `Ctrl-X-` (highlighted) |
| Quick help | Always | `F1:help  Ctrl-Q:quit` |

### Command/search prompt

When in ex-command mode (`:`) or search mode (`/`), the status bar is
replaced by the prompt:

```
 :s/foo/bar/g
```

```
 /search_pattern
```

## Editor panel

### Line numbers

Shown by default (`:set nonumber` to hide). Right-aligned, separated from
code by ` │ ` (always present, not a border).

```
  1 │ fn main() {
  2 │     let app = App::new();
  3 │     run_loop(&mut app);
  4 │ }
```

### Gutter

Between line numbers and code, a 1-character gutter shows:

- Git diff markers: `+` (added, green), `~` (modified, blue), `-` (deleted, red)
- Diagnostic markers: `●` (error, red), `▲` (warning, yellow)
- Breakpoint: `◉` (if debugging support added later)

```
  1   │ fn main() {
  2 + │     let app = App::new();
  3 ~ │     run_loop(&mut app);
  4   │ }
  5 ● │     let x = undefined_var;
```

### Cursor styles

| Mode | Cursor | Color |
|------|--------|-------|
| Normal (vim) | Block | Reverse video |
| Insert (vim) | Vertical bar | Bright |
| Visual (vim) | Block on selection | Selection bg |
| Emacs | Vertical bar | Bright |
| Classic | Underline | Bright |

### Whitespace display (`:set list`)

```
  1 │ fn·main()·{¶
  2 │ ──▶let·app·=·App::new();¶
  3 │ }¶
```

- Space → `·` (middle dot, dim)
- Tab → `──▶` (arrow spanning tab width, dim)
- EOL → `¶` (pilcrow, dim)
- Trailing whitespace → highlighted in error color

### Search highlights

All matches: yellow background. Current match: orange background.
Match count shown in status bar: `[3/17]`.

### Long lines

No wrapping by default. Horizontal scroll follows cursor. A subtle `»`
indicator at the right edge shows content continues. `:set wrap` enables
soft wrapping.

## Tree panel

### File tree mode

```
  📁 src/
    📁 editor/
      🦀 mod.rs
      🦀 vi.rs
      🦀 undo.rs
    🦀 main.rs
  📄 Cargo.toml
  📄 Makefile
  📄 README.md
```

- Directories: 📁 (expanded) / 📁 (collapsed, dimmer)
- Files: language icon (🦀 .rs, ☕ .java, 🔷 .ts, 🐹 .go, 📄 other)
- Git status colors: green (new), blue (modified), red (deleted), gray (ignored)
- Current file highlighted with cursor bar

### Git mode

Only shows modified/untracked/staged files, grouped:

```
  Staged:
    🦀 src/editor/mod.rs
  Modified:
    🦀 src/app.rs
    📄 Cargo.toml
  Untracked:
    📄 doc/f4-design/v-001-vision.md
```

### Packages mode (Java)

```
  com.example
    .service
      UserService
      OrderService
    .model
      User
      Order
    .controller
      UserController
```

### Symbols mode

```
  fn main()
  fn run_loop()
  struct App
    fn new()
    fn handle_key()
    fn render()
  enum PanelAction
```

## Bottom panel

### Tab bar

```
 ▸terminal  kiro  errors  search  tests  output
```

Active tab: bright text on colored background. Inactive: dim text.
`Ctrl-Shift-↑/↓` cycles tabs when bottom panel is focused.

### Terminal tab

Full terminal emulation (vte + PTY). Multiple sub-tabs for different
shells. Scrollback with PgUp/PgDn.

### Kiro tab

Same as terminal but running `kiro-cli chat`. Additional features:
- "Send to kiro" from editor injects context
- Code blocks in output are detected and offered as "apply" actions
- Diff blocks show inline accept/reject

### Errors tab

```
  ✕ src/main.rs:42:17  error[E0425]: cannot find value `x`
  ✕ src/app.rs:108:5   error[E0308]: mismatched types
  ▲ src/lib.rs:23:1    warning: unused import
```

Navigable: Enter jumps to file:line. F8/Shift-F8 for next/prev error.

### Search tab

```
  src/editor/mod.rs:45    pub fn cursor_row(&self) -> usize {
  src/editor/mod.rs:142   self.cursor_row = row;
  src/editor/vi.rs:87     let row = buf.cursor_row();
```

Navigable: Enter opens file at line.

### Tests tab

```
  ✅ editor::undo::tests (5/5)
  ✅ editor::vi::tests (27/27)
  ❌ nav::java::tests (3/4)
     ❌ resolve_nested_package — expected Some, got None
  ✅ content_search::tests (5/5)
```

Navigable: Enter on failure jumps to test source.

## Overlays

### Command palette

Centered overlay, fuzzy-searchable:

```
┌─ Commands ──────────────────────────────┐
│ > save                                  │
│   buffer save          Ctrl-S           │
│   buffer save-all      Ctrl-Shift-S     │
│   buffer save-as                        │
│   session save         Ctrl-X S         │
└─────────────────────────────────────────┘
```

### Fuzzy file search (Ctrl-P)

Same pattern as command palette but searches file paths.

### Completion popup

Appears near cursor, anchored below the current line:

```
    let app = App::n|
              ┌──────────────┐
              │ new()        │
              │ name()       │
              │ navigate()   │
              └──────────────┘
```

### Rename dialog

Inline: the symbol is highlighted and editable in-place. Preview of all
affected locations shown in a split below.

## Resize behavior

On terminal resize:
1. Grid is reallocated to new size
2. Layout engine recomputes panel areas
3. All panels redraw completely (no incremental)
4. PTY terminals are resized via `SIGWINCH` / pty resize
5. If terminal shrinks below minimum panel sizes, panels collapse in order:
   control → tree → bottom panel shrinks to minimum

## Makefile handling

When editing a Makefile:
- Tab characters are preserved (never converted to spaces)
- In `:set list` mode, tabs show as `──▶` with distinct color
- Tab key in insert mode always inserts a literal tab
- The `tab-width` setting controls display width but not insertion

## Binary files

Binary files show a hex dump view instead of text:

```
  00000000  7f 45 4c 46 02 01 01 00  00 00 00 00 00 00 00 00  |.ELF............|
  00000010  03 00 3e 00 01 00 00 00  40 10 40 00 00 00 00 00  |..>.....@.@.....|
```

Read-only. No editing.
