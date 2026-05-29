# Unified Inline Editing Design

## Problem

Two separate single-line editing implementations exist:
- **InlineEditor** (txv-widgets): plain struct, caller draws it, no View trait
- **InputLine** (txv-widgets): proper View, owns its buffer, emits commands

This causes inconsistent UX, duplicated bug fixes, and prevents shared features
(completion popup, history) from working everywhere.

## Solution

**InputLine becomes the single editing widget.** InlineEditor is deleted.

## Architecture

### Embedding Pattern

Any view that needs inline editing becomes a Group (with zero children normally).
When editing starts:

1. Create an InputLine, configure it (text, style, completer, history)
2. Position it at the edit row's screen coordinates
3. Run `exec_view(input_line)` — modal loop
4. On exit: read commit/cancel result, destroy InputLine

No persistent child management. No repositioning. The modal loop handles
everything until commit, cancel, or terminal resize.

### SidekickManager

A small component at the desktop level that handles popup placement:

- Receives commands: "show View at Rect" / "hide"
- Places the View into the desktop's draw cycle at translated coordinates
- Desktop draws it last (on top of everything)
- The sidekick View is a puppet — it never has focus, never handles keys directly

### InputLine ↔ Sidekick Interaction

InputLine owns the sidekick View instance and brokers all interaction:

- On text change: queries completer, updates sidekick content
- Sends "place sidekick at rect" command to SidekickManager
- Routes arrow keys to sidekick for navigation (separation of concerns)
- Routes sidekick commands back (selection picked → apply to text)
- On exit: sends "hide sidekick" command

### History = Completions

History is just another completion source. The sidekick shows a merged list
from all registered sources (history, path completer, command completer, etc.).
Up/Down with empty text shows history; typing filters completions.

### Terminal Resize During Modal

Resize event exits the modal loop as cancel (or prompts "confirm/cancel"
via status bar if text was modified). Simple and safe.

## InputLine Feature Additions

| Feature | Status |
|---------|--------|
| Overflow indicators ('…') | ✅ Done |
| Horizontal scroll | ✅ Done |
| Inherit-bg mode | TODO — preserve underlying cell bg |
| Shift+Arrow selection | TODO |
| select_all() | ✅ Exists |
| set_text() / clear() | ✅ Exists |
| Custom palette | ✅ Exists |
| Completer trait | ✅ Exists |
| History | ✅ Exists |

## Migration Order

1. Add inherit-bg + shift-selection to InputLine
2. Build SidekickManager + sidekick View
3. Convert todo_tree (simplest consumer)
4. Convert struct_view
5. Convert csv_view (also fix existing draw bug)
6. Delete InlineEditor + InlineEditDelegate
