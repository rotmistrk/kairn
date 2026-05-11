# Task: Port Chrome Visuals to New TabGroup

## Context

The desktop was rewritten from SlottedDesktop to TabGroup + LayoutGroup.
The new structure is correct but lost all visual polish. The old chrome
code exists at git commit `2a7e71c` in these files:
- `src/desktop/chrome.rs` — tab bar with Powerline glyphs, colors
- `src/desktop/dropdown.rs` — dropdown tab picker with borders
- `src/glyphs.rs` — glyph definitions (still exists, unchanged)

The new code is in:
- `txv-widgets/src/tab_group_view.rs` — minimal plain chrome (REPLACE)

## Exit Criteria

1. Tab bar looks IDENTICAL to the old version:
   - Powerline glyphs (▶◀ rounded caps from `src/glyphs.rs`)
   - Focused tab: cyan fg, blue bg, bold
   - Unfocused tab: white fg, dark gray bg
   - Tab count badge: `❨N❩` in cyan bg
   - Chrome line fills remaining width with `─`

2. Dropdown tab picker works:
   - Opens on Ctrl-Shift-Down
   - Shows numbered entries with borders (│ ╰ ─ ╯)
   - Cursor highlighted (bold cyan)
   - Scrolls when entries exceed height
   - Digit keys select directly
   - Esc closes

3. Git status colors in file tree preserved:
   - Directories: light blue (Ansi 14)
   - Modified: blue (Ansi 12)
   - Added: green (Ansi 2)
   - Untracked: red (Ansi 1)
   - Conflict: magenta (Ansi 5)
   - Cursor: blue bg (Ansi 4) + underline when focused

4. Editor dirty marker: `*filename` when unsaved

5. ALL existing tests pass (cargo test --workspace --no-fail-fast)

6. Pre-commit hook passes (bash hooks/pre-commit)

## How To Do It

### Step 1: Get the old chrome code

```bash
git show 2a7e71c:src/desktop/chrome.rs > /tmp/old_chrome.rs
git show 2a7e71c:src/desktop/dropdown.rs > /tmp/old_dropdown.rs
```

### Step 2: Port draw_chrome to tab_group_view.rs

Replace the current minimal `draw_chrome` in `txv-widgets/src/tab_group_view.rs`
with the rendering logic from old_chrome.rs. Adapt:
- Old code used `self.slots[sid]` → new code uses `self.titles[i]` and `self.group.focused`
- Old code used `self.display_name(slot, idx)` → new code uses `self.titles[i]`
- Old code used `self.focused == sid` → new code uses `i == self.group.focused`
- Import `crate::glyphs::glyphs` (or copy glyph constants locally)

The style functions to port:
- `chrome_style()` → gray on black
- `focused_title()` → cyan on blue, bold
- `focused_arrow()` → green on blue
- `focused_count()` → white on cyan, bold
- `active_title()` → white on dark gray, bold
- `active_arrow()` → gray on dark gray
- `active_count()` → white on dark gray

### Step 3: Port dropdown

Add dropdown state to TabGroup:
```rust
dropdown_open: bool,
dropdown_cursor: usize,
```

Port the dropdown drawing from old_dropdown.rs into a `draw_dropdown` method.
Port the dropdown key handling (Esc, Enter, Up/Down, digits).

### Step 4: Verify colors

Check that `src/views/tree.rs` still applies git status colors.
Check that `src/views/editor/mod.rs` still shows `*` for dirty files.
Check that tree cursor uses blue bg + underline when focused.

### Step 5: Write tests

Add to `tests/chrome.rs` (or create new test file):

```rust
#[test]
fn tab_bar_shows_powerline_glyphs() {
    // Check that row 0 contains the Powerline glyph chars
}

#[test]
fn focused_tab_has_blue_background() {
    // Check cell style at the focused tab position
}

#[test]
fn dropdown_shows_numbered_entries() {
    // Open dropdown, check entries visible with borders
}

#[test]
fn dropdown_digit_selects_tab() {
    // Open dropdown, press '1', verify tab switched
}

#[test]
fn dirty_editor_shows_asterisk_in_title() {
    // Edit file, check tab title starts with *
}
```

## Constraints

- Pre-commit hook MUST pass
- 240 code line max per file (split if needed)
- No unwrap/expect/panic
- The glyphs.rs file already exists — USE IT
- Do NOT change the LayoutGroup or TabGroup structure — only the VISUAL code

## Reference: Old Style Functions (from 2a7e71c:src/desktop/chrome.rs)

```rust
fn chrome_style() -> Style {
    Style { fg: Color::Ansi(7), bg: Color::Ansi(0), attrs: Attrs::default() }
}
fn focused_title() -> Style {
    Style { fg: Color::Ansi(14), bg: Color::Ansi(4), attrs: Attrs { bold: true, ..Attrs::default() } }
}
fn focused_arrow() -> Style {
    Style { fg: Color::Ansi(10), bg: Color::Ansi(4), attrs: Attrs::default() }
}
fn focused_count() -> Style {
    Style { fg: Color::Ansi(15), bg: Color::Ansi(6), attrs: Attrs { bold: true, ..Attrs::default() } }
}
fn active_title() -> Style {
    Style { fg: Color::Ansi(15), bg: Color::Ansi(8), attrs: Attrs { bold: true, ..Attrs::default() } }
}
fn active_arrow() -> Style {
    Style { fg: Color::Ansi(7), bg: Color::Ansi(8), attrs: Attrs::default() }
}
fn active_count() -> Style {
    Style { fg: Color::Ansi(15), bg: Color::Ansi(8), attrs: Attrs::default() }
}
```
