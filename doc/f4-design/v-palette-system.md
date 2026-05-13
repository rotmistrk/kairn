# Palette System Design

**Status:** Proposed  
**Scope:** txv-core, txv-widgets, kairn  
**Problem:** 73 hardcoded `Color::Ansi` usages across 15 files with zero style constants. Colors are consistent by convention only — no single source of truth, no user configurability.

**Goal:** A semantic palette system that:
1. Names every visual role once
2. Provides sensible defaults matching current behavior
3. Allows partial user override via `.kairnrc`
4. Separates framework concerns (txv) from app concerns (kairn)

---

## 1. Semantic Role Taxonomy

All 73+ hardcoded styles categorized into a hierarchical namespace:

### Base

| Role | Description | Current Usage |
|------|-------------|---------------|
| `base.text` | Default foreground | `Color::Reset` (terminal default) |
| `base.background` | Default background | `Color::Reset` (terminal default) |
| `base.dim` | Secondary/muted text | `fg: Ansi(8)` — gutter, hints, separators, done items |
| `base.bright` | Emphasized text | `fg: Ansi(15)` — dropdown text, bright labels |
| `base.border` | Dialog/window borders | `bold: true` |
| `base.separator` | Structural dividers | `fg: Ansi(8)` — split panes, tab bar lines, scrollbar track |

### Interactive

| Role | Description | Current Usage |
|------|-------------|---------------|
| `interactive.cursor_focused` | Selected item in focused view | `bg: Ansi(4), underline: true` |
| `interactive.cursor_unfocused` | Selected item in unfocused view | `bg: Ansi(8)` |
| `interactive.input_cursor` | Text input caret | `reverse: true` |
| `interactive.edit_overlay` | Inline edit mode | `fg: Ansi(0), bg: Ansi(3)` |
| `interactive.search_match` | Search hit highlight | `bg: Ansi(3)` |
| `interactive.visual_selection` | Vi visual mode | `fg: Ansi(3), reverse: true` |
| `interactive.disabled` | Greyed-out items | `fg: Ansi(8)` |

### Chrome

| Role | Description | Current Usage |
|------|-------------|---------------|
| `chrome.bar` | Chrome bar background | `fg: Ansi(7), bg: Ansi(0)` |
| `chrome.tab_focused` | Focused panel tab title | `fg: Ansi(14), bg: Ansi(4), bold` |
| `chrome.tab_focused_arrow` | Powerline arrow (focused) | `fg: Ansi(10), bg: Ansi(4)` |
| `chrome.tab_focused_badge` | Count badge (focused) | `fg: Ansi(15), bg: Ansi(6), bold` |
| `chrome.tab_active` | Active (unfocused) tab title | `fg: Ansi(15), bg: Ansi(8), bold` |
| `chrome.tab_active_arrow` | Powerline arrow (active) | `fg: Ansi(7), bg: Ansi(8)` |
| `chrome.tab_active_badge` | Count badge (active) | `fg: Ansi(15), bg: Ansi(8)` |
| `chrome.status_bar` | Status bar | `reverse: true` |
| `chrome.progress_bar` | Progress fill | `reverse: true` |
| `chrome.scrollbar_track` | Scrollbar track | `fg: Ansi(8)` |
| `chrome.scrollbar_thumb` | Scrollbar thumb | `reverse: true` |

### Popup

| Role | Description | Current Usage |
|------|-------------|---------------|
| `popup.background` | Popup/dropdown bg | `fg: Ansi(15), bg: Ansi(0)` |
| `popup.border` | Popup border | `fg: Ansi(6), bg: Ansi(0)` |
| `popup.selected` | Selected popup item | `fg: Ansi(15), bg: Ansi(4), underline` |
| `popup.table_header` | Table header row | `bold: true, reverse: true` |

### State

| Role | Description | Current Usage |
|------|-------------|---------------|
| `state.error` | Error text | `fg: Ansi(1)` or `fg: Ansi(9)` |
| `state.warning` | Warning text | `fg: Ansi(3)` or `fg: Ansi(11)` |
| `state.info` | Info text | `fg: Ansi(6)` |
| `state.success` | Success/done text | `fg: Ansi(2)` or `fg: Ansi(10)` |
| `state.hint` | Hint/subtle diagnostic | `fg: Ansi(8)` |

### Domain: Git

| Role | Description | Current Usage |
|------|-------------|---------------|
| `git.added` | Added/staged file | `fg: Ansi(2)` |
| `git.modified` | Modified file | `fg: Ansi(12)` |
| `git.untracked` | Untracked file | `fg: Ansi(1)` |
| `git.ignored` | Ignored file | `fg: Ansi(8)` |
| `git.conflict` | Conflict file | `fg: Ansi(5)` |
| `git.clean` | Clean/normal file | `fg: Ansi(7)` |

### Domain: Diff

| Role | Description | Current Usage |
|------|-------------|---------------|
| `diff.added` | Added line | `fg: Ansi(2)` |
| `diff.deleted` | Deleted line | `fg: Ansi(1)` |
| `diff.fold` | Context fold separator | `fg: Ansi(8)` |

### Domain: Editor

| Role | Description | Current Usage |
|------|-------------|---------------|
| `editor.gutter` | Line numbers | `fg: Ansi(8)` |
| `editor.list_chars` | Whitespace indicators | `fg: Ansi(8)` |
| `editor.cursor` | Block cursor | `reverse: true` |

### Domain: Diagnostics

| Role | Description | Current Usage |
|------|-------------|---------------|
| `diag.error` | Error underline | `fg: Ansi(1), underline` |
| `diag.warning` | Warning underline | `fg: Ansi(3), underline` |
| `diag.info` | Info underline | `fg: Ansi(6), underline` |
| `diag.hint` | Hint underline | `fg: Ansi(8), underline` |

### Domain: Tree

| Role | Description | Current Usage |
|------|-------------|---------------|
| `tree.directory` | Directory name | `fg: Ansi(14)` |

### Domain: Todo

| Role | Description | Current Usage |
|------|-------------|---------------|
| `todo.normal` | Normal item | `fg: Ansi(7)` |
| `todo.done` | Completed item | `fg: Ansi(8)` |
| `todo.important` | Important item | `fg: Ansi(1)` |

### Domain: Messages

| Role | Description | Current Usage |
|------|-------------|---------------|
| `msg.error` | Error message | `fg: Ansi(9)` |
| `msg.warning` | Warning message | `fg: Ansi(11)` |
| `msg.info` | Info message | `fg: Ansi(7)` |
| `msg.debug` | Debug message | `fg: Ansi(8)` |

### Domain: Welcome

| Role | Description | Current Usage |
|------|-------------|---------------|
| `welcome.logo` | ASCII art | `fg: Ansi(14)` |
| `welcome.hint` | Help text | `fg: Ansi(8)` |

---

## 2. Proposed Defaults

Complete palette using only Ansi(0–15) + Reset. Matches current behavior exactly.

| Role | fg | bg | attrs |
|------|----|----|-------|
| `base.text` | Reset | Reset | — |
| `base.dim` | 8 | — | — |
| `base.bright` | 15 | — | — |
| `base.border` | Reset | — | bold |
| `base.separator` | 8 | — | — |
| `interactive.cursor_focused` | — | 4 | underline |
| `interactive.cursor_unfocused` | — | 8 | — |
| `interactive.input_cursor` | — | — | reverse |
| `interactive.edit_overlay` | 0 | 3 | — |
| `interactive.search_match` | — | 3 | — |
| `interactive.visual_selection` | 3 | — | reverse |
| `interactive.disabled` | 8 | — | — |
| `chrome.bar` | 7 | 0 | — |
| `chrome.tab_focused` | 14 | 4 | bold |
| `chrome.tab_focused_arrow` | 10 | 4 | — |
| `chrome.tab_focused_badge` | 15 | 6 | bold |
| `chrome.tab_active` | 15 | 8 | bold |
| `chrome.tab_active_arrow` | 7 | 8 | — |
| `chrome.tab_active_badge` | 15 | 8 | — |
| `chrome.status_bar` | — | — | reverse |
| `chrome.progress_bar` | — | — | reverse |
| `chrome.scrollbar_track` | 8 | — | — |
| `chrome.scrollbar_thumb` | — | — | reverse |
| `popup.background` | 15 | 0 | — |
| `popup.border` | 6 | 0 | — |
| `popup.selected` | 15 | 4 | underline |
| `popup.table_header` | — | — | bold, reverse |
| `state.error` | 1 | — | — |
| `state.warning` | 3 | — | — |
| `state.info` | 6 | — | — |
| `state.success` | 2 | — | — |
| `state.hint` | 8 | — | — |
| `git.added` | 2 | — | — |
| `git.modified` | 12 | — | — |
| `git.untracked` | 1 | — | — |
| `git.ignored` | 8 | — | — |
| `git.conflict` | 5 | — | — |
| `git.clean` | 7 | — | — |
| `diff.added` | 2 | — | — |
| `diff.deleted` | 1 | — | — |
| `diff.fold` | 8 | — | — |
| `editor.gutter` | 8 | — | — |
| `editor.list_chars` | 8 | — | — |
| `editor.cursor` | — | — | reverse |
| `diag.error` | 1 | — | underline |
| `diag.warning` | 3 | — | underline |
| `diag.info` | 6 | — | underline |
| `diag.hint` | 8 | — | underline |
| `tree.directory` | 14 | — | — |
| `todo.normal` | 7 | — | — |
| `todo.done` | 8 | — | — |
| `todo.important` | 1 | — | — |
| `msg.error` | 9 | — | — |
| `msg.warning` | 11 | — | — |
| `msg.info` | 7 | — | — |
| `msg.debug` | 8 | — | — |
| `welcome.logo` | 14 | — | — |
| `welcome.hint` | 8 | — | — |

**Design note:** `state.*` uses dim colors (1, 3, 6) for inline diagnostics; `msg.*` uses bright variants (9, 11) for status bar messages where visibility matters more.

---

## 3. Data Structure (Rust)

### Core Style Type (already exists in txv-core)

```rust
// txv-core/src/cell.rs — existing
#[derive(Clone, Copy, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub underline: bool,
    pub reverse: bool,
}
```

### Palette Style Entry (new — supports partial override)

```rust
// txv-core/src/palette.rs

use serde::{Deserialize, Serialize};

/// A single palette entry. All fields are Option to support partial overlay.
/// `None` means "inherit from base.text" (or leave unchanged).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PaletteStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fg: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
}

impl PaletteStyle {
    /// Resolve to a concrete Style, filling unset fields from `base`.
    pub fn resolve(&self, base: &Style) -> Style {
        Style {
            fg: self.fg.unwrap_or(base.fg),
            bg: self.bg.unwrap_or(base.bg),
            bold: self.bold.unwrap_or(base.bold),
            underline: self.underline.unwrap_or(base.underline),
            reverse: self.reverse.unwrap_or(base.reverse),
        }
    }

    /// Merge an overlay on top of self (overlay wins where set).
    pub fn merge(&self, overlay: &PaletteStyle) -> PaletteStyle {
        PaletteStyle {
            fg: overlay.fg.or(self.fg),
            bg: overlay.bg.or(self.bg),
            bold: overlay.bold.or(self.bold),
            underline: overlay.underline.or(self.underline),
            reverse: overlay.reverse.or(self.reverse),
        }
    }
}
```

### Framework Palette (txv-core)

```rust
// txv-core/src/palette.rs

/// Framework-level palette — roles that any txv app needs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Palette {
    pub base: BasePalette,
    pub interactive: InteractivePalette,
    pub chrome: ChromePalette,
    pub popup: PopupPalette,
    pub state: StatePalette,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BasePalette {
    pub text: PaletteStyle,
    pub dim: PaletteStyle,
    pub bright: PaletteStyle,
    pub border: PaletteStyle,
    pub separator: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InteractivePalette {
    pub cursor_focused: PaletteStyle,
    pub cursor_unfocused: PaletteStyle,
    pub input_cursor: PaletteStyle,
    pub edit_overlay: PaletteStyle,
    pub search_match: PaletteStyle,
    pub visual_selection: PaletteStyle,
    pub disabled: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChromePalette {
    pub bar: PaletteStyle,
    pub tab_focused: PaletteStyle,
    pub tab_focused_arrow: PaletteStyle,
    pub tab_focused_badge: PaletteStyle,
    pub tab_active: PaletteStyle,
    pub tab_active_arrow: PaletteStyle,
    pub tab_active_badge: PaletteStyle,
    pub status_bar: PaletteStyle,
    pub progress_bar: PaletteStyle,
    pub scrollbar_track: PaletteStyle,
    pub scrollbar_thumb: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PopupPalette {
    pub background: PaletteStyle,
    pub border: PaletteStyle,
    pub selected: PaletteStyle,
    pub table_header: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatePalette {
    pub error: PaletteStyle,
    pub warning: PaletteStyle,
    pub info: PaletteStyle,
    pub success: PaletteStyle,
    pub hint: PaletteStyle,
}
```

### Default Implementation

```rust
impl Default for Palette {
    fn default() -> Self {
        Self {
            base: BasePalette {
                text: PaletteStyle::default(), // Reset/Reset
                dim: PaletteStyle { fg: Some(Ansi(8)), ..Default::default() },
                bright: PaletteStyle { fg: Some(Ansi(15)), ..Default::default() },
                border: PaletteStyle { bold: Some(true), ..Default::default() },
                separator: PaletteStyle { fg: Some(Ansi(8)), ..Default::default() },
            },
            interactive: InteractivePalette {
                cursor_focused: PaletteStyle {
                    bg: Some(Ansi(4)), underline: Some(true), ..Default::default()
                },
                cursor_unfocused: PaletteStyle {
                    bg: Some(Ansi(8)), ..Default::default()
                },
                input_cursor: PaletteStyle {
                    reverse: Some(true), ..Default::default()
                },
                edit_overlay: PaletteStyle {
                    fg: Some(Ansi(0)), bg: Some(Ansi(3)), ..Default::default()
                },
                search_match: PaletteStyle {
                    bg: Some(Ansi(3)), ..Default::default()
                },
                visual_selection: PaletteStyle {
                    fg: Some(Ansi(3)), reverse: Some(true), ..Default::default()
                },
                disabled: PaletteStyle {
                    fg: Some(Ansi(8)), ..Default::default()
                },
            },
            chrome: ChromePalette {
                bar: PaletteStyle {
                    fg: Some(Ansi(7)), bg: Some(Ansi(0)), ..Default::default()
                },
                tab_focused: PaletteStyle {
                    fg: Some(Ansi(14)), bg: Some(Ansi(4)), bold: Some(true),
                    ..Default::default()
                },
                tab_focused_arrow: PaletteStyle {
                    fg: Some(Ansi(10)), bg: Some(Ansi(4)), ..Default::default()
                },
                tab_focused_badge: PaletteStyle {
                    fg: Some(Ansi(15)), bg: Some(Ansi(6)), bold: Some(true),
                    ..Default::default()
                },
                tab_active: PaletteStyle {
                    fg: Some(Ansi(15)), bg: Some(Ansi(8)), bold: Some(true),
                    ..Default::default()
                },
                tab_active_arrow: PaletteStyle {
                    fg: Some(Ansi(7)), bg: Some(Ansi(8)), ..Default::default()
                },
                tab_active_badge: PaletteStyle {
                    fg: Some(Ansi(15)), bg: Some(Ansi(8)), ..Default::default()
                },
                status_bar: PaletteStyle {
                    reverse: Some(true), ..Default::default()
                },
                progress_bar: PaletteStyle {
                    reverse: Some(true), ..Default::default()
                },
                scrollbar_track: PaletteStyle {
                    fg: Some(Ansi(8)), ..Default::default()
                },
                scrollbar_thumb: PaletteStyle {
                    reverse: Some(true), ..Default::default()
                },
            },
            popup: PopupPalette {
                background: PaletteStyle {
                    fg: Some(Ansi(15)), bg: Some(Ansi(0)), ..Default::default()
                },
                border: PaletteStyle {
                    fg: Some(Ansi(6)), bg: Some(Ansi(0)), ..Default::default()
                },
                selected: PaletteStyle {
                    fg: Some(Ansi(15)), bg: Some(Ansi(4)), underline: Some(true),
                    ..Default::default()
                },
                table_header: PaletteStyle {
                    bold: Some(true), reverse: Some(true), ..Default::default()
                },
            },
            state: StatePalette {
                error: PaletteStyle { fg: Some(Ansi(1)), ..Default::default() },
                warning: PaletteStyle { fg: Some(Ansi(3)), ..Default::default() },
                info: PaletteStyle { fg: Some(Ansi(6)), ..Default::default() },
                success: PaletteStyle { fg: Some(Ansi(2)), ..Default::default() },
                hint: PaletteStyle { fg: Some(Ansi(8)), ..Default::default() },
            },
        }
    }
}
```

### App-Specific Extension (kairn)

```rust
// kairn/src/palette.rs

/// kairn-specific palette roles that extend the framework palette.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppPalette {
    #[serde(flatten)]
    pub base: Palette,       // framework roles
    pub git: GitPalette,
    pub diff: DiffPalette,
    pub editor: EditorPalette,
    pub diag: DiagPalette,
    pub tree: TreePalette,
    pub todo: TodoPalette,
    pub msg: MsgPalette,
    pub welcome: WelcomePalette,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitPalette {
    pub added: PaletteStyle,
    pub modified: PaletteStyle,
    pub untracked: PaletteStyle,
    pub ignored: PaletteStyle,
    pub conflict: PaletteStyle,
    pub clean: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiffPalette {
    pub added: PaletteStyle,
    pub deleted: PaletteStyle,
    pub fold: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorPalette {
    pub gutter: PaletteStyle,
    pub list_chars: PaletteStyle,
    pub cursor: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiagPalette {
    pub error: PaletteStyle,
    pub warning: PaletteStyle,
    pub info: PaletteStyle,
    pub hint: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreePalette {
    pub directory: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoPalette {
    pub normal: PaletteStyle,
    pub done: PaletteStyle,
    pub important: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MsgPalette {
    pub error: PaletteStyle,
    pub warning: PaletteStyle,
    pub info: PaletteStyle,
    pub debug: PaletteStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WelcomePalette {
    pub logo: PaletteStyle,
    pub hint: PaletteStyle,
}
```

---

## 4. Instrumentation Plan

### Phase 1: Define (txv-core)

1. Add `palette.rs` to txv-core with `PaletteStyle`, `Palette`, and sub-structs.
2. Add `palette: &Palette` field to `Program` (the event loop owner).
3. Pass `&Palette` through `DrawContext` so every `View::draw()` can access it.

```rust
// txv-core/src/draw_context.rs
pub struct DrawContext<'a> {
    pub surface: &'a mut Surface,
    pub palette: &'a Palette,
}
```

### Phase 2: Wire (txv-widgets)

Replace each hardcoded style with a palette lookup. Example migration:

```rust
// BEFORE (list_view.rs)
let style = Style { bg: Color::Ansi(4), underline: true, ..cell_style };

// AFTER
let style = ctx.palette.interactive.cursor_focused.resolve(&cell_style);
```

Each widget reads from the appropriate palette field. No widget owns colors.

### Phase 3: Extend (kairn)

1. `AppPalette` wraps `Palette` + app-specific sub-palettes.
2. On startup, load `AppPalette::default()`, then overlay from `.kairnrc`.
3. Store `AppPalette` in the application struct, pass `&palette.base` to txv widgets via `Program`, and use `&palette` directly in kairn views.

### Phase 4: Config Loading

```rust
// Partial overlay from .kairnrc — only set fields override defaults
fn load_palette(config: &Value) -> AppPalette {
    let mut palette = AppPalette::default();
    if let Some(theme) = config.get("theme") {
        // serde partial: deserialize into Option fields, merge
        let overlay: AppPalette = serde_json::from_value(theme.clone())
            .unwrap_or_default();
        palette.merge_overlay(&overlay);
    }
    palette
}
```

### Config Format (JSON)

```json
{
  "theme": {
    "interactive": {
      "cursor_focused": { "bg": 4, "underline": true },
      "edit_overlay": { "fg": 0, "bg": 3 }
    },
    "git": {
      "modified": { "fg": 11 }
    }
  }
}
```

Only specified fields override. Everything else keeps defaults. This matches the existing `.kairnrc` sparse overlay pattern used for keybindings.

### Migration Strategy

Migrate one widget at a time. Each PR:
1. Replace hardcoded styles in one widget with palette lookups
2. Verify visual output is identical (no behavior change)
3. Tests continue to pass

Order: ListView → TreeView → Table → StatusBar → Dialog → Editor views → Chrome

---

## 5. Separation of Concerns

### txv-core owns:

| Category | Roles |
|----------|-------|
| Base | text, dim, bright, border, separator |
| Interactive | cursor_focused, cursor_unfocused, input_cursor, edit_overlay, search_match, visual_selection, disabled |
| Chrome | bar, tab_focused, tab_active, status_bar, progress_bar, scrollbar_track, scrollbar_thumb, tab arrows/badges |
| Popup | background, border, selected, table_header |
| State | error, warning, info, success, hint |

**Rationale:** These are generic TUI concerns. Any app built on txv needs selection colors, status bars, and state indicators.

### kairn owns:

| Category | Roles |
|----------|-------|
| Git | added, modified, untracked, ignored, conflict, clean |
| Diff | added, deleted, fold |
| Editor | gutter, list_chars, cursor |
| Diagnostics | error, warning, info, hint (with underline) |
| Tree | directory |
| Todo | normal, done, important |
| Messages | error, warning, info, debug |
| Welcome | logo, hint |

**Rationale:** These are domain-specific to an IDE. A different txv app (e.g., a file manager) would have different domain roles.

### Extension Mechanism

```rust
pub struct AppPalette {
    pub base: Palette,          // txv-core's palette (passed to framework)
    pub git: GitPalette,        // app-specific extensions
    pub diff: DiffPalette,
    // ...
}
```

kairn views access `app_palette.diff.added` directly. When calling into txv widgets, pass `&app_palette.base`. The framework never knows about git or diagnostics.

### Inheritance Chain

```
Terminal defaults (Reset/Reset)
  └── base.text (user's terminal theme)
       ├── base.dim (inherits bg from text)
       ├── interactive.cursor_focused (inherits fg from text)
       ├── state.error (inherits bg from text)
       └── git.added (inherits bg from text)
```

`PaletteStyle::resolve(&base_style)` implements this: unset fields fall through to the base style passed by the caller.

---

## 6. Prior Art Notes

### Turbo Vision CRT Palette

TV used a flat byte array indexed by widget ID × attribute slot:
```pascal
CPalette = array[0..255] of Byte;  // index → BIOS color attribute
```

**Pros:** Extremely fast lookup (array index). Compact.  
**Cons:** Opaque (what is slot 47?). No semantic names. Hard to extend. Fixed 16-color only.

**Our approach differs:** Named fields instead of numeric indices. Compile-time safety — you can't accidentally use `git.added` where `diff.added` is expected.

### X Resources (Xdefaults)

Hierarchical string keys with wildcard inheritance:
```
*foreground: white
XTerm*color4: #5555ff
URxvt.keysym.M-x: command
```

**Pros:** Infinite extensibility. Partial override is natural.  
**Cons:** No type safety. Typos silently ignored. Runtime string parsing.

**Our approach borrows:** The hierarchical naming (`chrome.tab_focused`) and partial override concept. But we use typed structs — a typo in `.kairnrc` produces a serde error, not silent failure.

### Helix Themes

TOML files with scoped keys:
```toml
[palette]
red = "#ff0000"

"ui.cursor.primary" = { fg = "red", modifiers = ["reversed"] }
"ui.statusline" = { fg = "white", bg = "blue" }
"diff.plus" = "green"
```

**Pros:** Clean separation of color definitions from role assignments. Full RGB support. Community themes.  
**Cons:** Flat string keys (no compile-time validation). Large theme files (~200 entries).

**Our approach borrows:** Semantic role names, separation of "what color" from "where used." We skip the indirection layer (named palette colors) for now — Ansi(0-15) are already named by the terminal theme. Can add later if RGB support is needed.

### Zellij

Rust structs with serde, KDL config format:
```kdl
themes {
    dracula {
        fg "#F8F8F2"
        bg "#282A36"
        red "#FF5555"
    }
}
```

**Pros:** Typed Rust structs. Named color slots.  
**Cons:** Flat (no hierarchy). Limited role vocabulary (just base colors, not semantic roles).

**Our approach differs:** Full semantic hierarchy rather than just base color names. Widgets reference roles (`interactive.cursor_focused`), not raw colors (`blue`).

---

## Summary

| Aspect | Decision |
|--------|----------|
| Color space | Ansi(0–15) only (terminal theme provides actual RGB) |
| Naming | Hierarchical dot-notation, grouped by concern |
| Storage | Typed Rust structs with `Option` fields for partial override |
| Serialization | serde JSON, matching `.kairnrc` pattern |
| Inheritance | `resolve(&base_style)` — unset fields fall through |
| Framework boundary | txv-core owns generic TUI roles; kairn owns domain roles |
| Migration | One widget at a time, visual parity verified per PR |
| Syntax highlighting | Out of scope (syntect themes are orthogonal) |
| Terminal emulation | Out of scope (VTE colors are dynamic, not themed) |

---

## 7. Dark/Light Mode Support

### Config: Dual Palettes

The `.kairnrc` `"theme"` key holds both variants:

```json
{
  "theme": {
    "mode": "auto",
    "dark": {
      "interactive": { "cursor_focused": { "bg": 4 } },
      "git": { "added": { "fg": 2 } }
    },
    "light": {
      "interactive": { "cursor_focused": { "bg": 12 } },
      "base": { "dim": { "fg": 7 } },
      "git": { "added": { "fg": 22 } }
    }
  }
}
```

| `mode` value | Behavior |
|--------------|----------|
| `"dark"` | Always use dark palette |
| `"light"` | Always use light palette |
| `"auto"` | Detect from system/terminal preference at startup |

Both `dark` and `light` are partial overlays on top of their respective
built-in defaults (`AppPalette::default_dark()` / `AppPalette::default_light()`).

### Data Structure

```rust
/// Theme configuration with dark/light variants.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_mode")]
    pub mode: ThemeMode,
    #[serde(default)]
    pub dark: AppPalette,
    #[serde(default)]
    pub light: AppPalette,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Dark,
    Light,
    Auto,
}

fn default_mode() -> ThemeMode { ThemeMode::Auto }
```

### Runtime State

```rust
/// Held in AppState — the active palette + both variants for hot-swap.
pub struct ThemeState {
    pub active: AppPalette,       // currently in use
    pub dark: AppPalette,         // resolved dark (defaults + user overlay)
    pub light: AppPalette,        // resolved light (defaults + user overlay)
    pub mode: ThemeMode,          // current mode
}

impl ThemeState {
    pub fn toggle(&mut self) {
        match self.mode {
            ThemeMode::Dark => {
                self.mode = ThemeMode::Light;
                self.active = self.light.clone();
            }
            _ => {
                self.mode = ThemeMode::Dark;
                self.active = self.dark.clone();
            }
        }
    }
}
```

### System Preference Detection

Detect terminal background at startup using the OSC 11 query:

```rust
/// Query terminal for background color via OSC 11.
/// Returns Dark if bg luminance < 0.5, Light otherwise.
/// Falls back to Dark if query times out (100ms).
pub fn detect_system_theme() -> ThemeMode {
    // Send: ESC ] 11 ; ? BEL
    // Expect: ESC ] 11 ; rgb:RRRR/GGGG/BBBB ST
    // Parse RGB, compute luminance, threshold at 0.5
    //
    // Timeout: 100ms — some terminals don't respond.
    // Fallback: ThemeMode::Dark (most developer terminals are dark).
}
```

This runs once at startup before entering raw mode. The result is cached.
If the terminal doesn't respond (tmux, some SSH), defaults to dark.

Alternative detection (macOS): read `defaults read -g AppleInterfaceStyle`
which returns "Dark" or empty. Can be checked as a fallback if OSC 11 fails.

### Hot-Swap Keybinding

| Key | Command | Action |
|-----|---------|--------|
| (configurable, default: none) | `CM_TOGGLE_THEME` | Swap dark↔light |
| M-x `theme dark` | — | Force dark |
| M-x `theme light` | — | Force light |
| M-x `theme auto` | — | Re-detect from system |

On toggle:
1. `ThemeState::toggle()` swaps the active palette
2. Emit `CM_REPAINT` to force full redraw
3. All widgets pick up new colors on next draw (they read from `&palette`)

No per-widget notification needed — the palette reference is shared,
and `CM_REPAINT` ensures everything redraws.

### Built-in Defaults

```rust
impl AppPalette {
    pub fn default_dark() -> Self {
        // Current defaults (Ansi 0-15 optimized for dark terminals)
        Self::default()
    }

    pub fn default_light() -> Self {
        // Adjusted for light backgrounds:
        // - dim: fg 7 (not 8, which is invisible on white)
        // - cursor_focused: bg 12 (bright blue, visible on white)
        // - cursor_unfocused: bg 7 (light gray)
        // - edit_overlay: fg 0, bg 11 (bright yellow)
        // - chrome.bar: fg 0, bg 7
        // - popup.background: fg 0, bg 15
        // - git.modified: fg 4 (dark blue, readable on white)
        Self { /* ... adjusted values ... */ }
    }
}
```

### LOE Addition

Dark/light mode adds ~2 hours:
- `ThemeConfig` / `ThemeState` structs: 30 min
- OSC 11 detection (with macOS fallback): 45 min
- `CM_TOGGLE_THEME` + M-x commands: 30 min
- Light palette defaults tuning: 15 min

**Revised total: ~12-15 hours.**

### Ownership Split: txv-core vs kairn

| Concern | txv-core | kairn |
|---------|----------|-------|
| Palette struct (framework roles) | `Palette` | `AppPalette` (wraps `Palette` + domain) |
| Dark defaults | `Palette::default_dark()` | `AppPalette::default_dark()` (calls txv + adds domain) |
| Light defaults | `Palette::default_light()` | `AppPalette::default_light()` (same) |
| Mode enum | `ThemeMode { Dark, Light, Auto }` | — (re-exports from txv-core) |
| System detection | `detect_system_theme() -> ThemeMode` | — (calls txv-core) |
| DrawContext plumbing | `DrawContext { palette: &Palette }` | passes `&app_palette.base` to framework |
| Config file format | — | `ThemeConfig { mode, dark, light }` |
| Runtime swap | — | `ThemeState { active, dark, light }` |
| Toggle command | — | `CM_TOGGLE_THEME` + M-x `theme` |

**Rationale:** Any txv app needs dark/light awareness. A file manager, a
dashboard, or a game built on txv all benefit from `detect_system_theme()`
and framework-level light defaults. Domain roles (git, diagnostics) are
kairn's concern — other apps would define their own extensions.
