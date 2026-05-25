# Changelog: migrate-tab-panel

## Architecture

The workspace system was completely rewritten. The old `LayoutGroup` + `TabGroup`
+ `ToolsPanel` hierarchy is replaced by composable primitives from txv-widgets:

- **TabPanel** — tab bar + content area (replaces TabGroup)
- **SplitPanel** — proportional split container with runtime direction switch
- **TiledWorkspace** — layout engine with wide/narrow modes, panel hide/show, resize

The old `Desktop` struct is eliminated. `TiledWorkspace` is now the top-level view.

## Breaking Changes

- Session schema bumped to v3 (backward-compatible via serde defaults)
- `SlotId::Right` / `SlotId::Bottom` merged into `SlotId::Tools`
- `EditorSplit` removed — native subpanels via `split_in_place()`
- Command IDs rebased to `CM_TXV_MAX + 1` to avoid collisions

## New Features

- **Subpanel splits**: `:split`, `:vsplit`, `:only`, `Ctrl-W` prefix bindings
- **Side-by-side diff**: `:diff -y` with aligned gap rendering
- **Panel resize**: `Alt-Shift-←/→/↑/↓`, `Alt-=/Alt--` (grow/shrink)
- **Layout cycling**: `Alt-\` cycles auto/wide/tall
- **Panel toggle**: `Alt-,` (tree), `Alt-.` (tools)
- **Tab management**: `Alt-0` dropdown, `Alt-;/Alt-'` next/prev, `Alt-w` close
- **Session persistence**: saves/restores panel proportions, splits, hidden state
- **LSP preamble discovery**: shebang-based `--lsp-preamble` for Tcl scripts
- **Kiro session management**: resume sessions, registry tracking
- **Autosave race fix**: flush dirty buffers before quit/close
- **OSC title sync**: shell tabs update title from terminal subtitle

## New Scripting Commands

- `view theme|zoom|toggle-tree|toggle-tools|layout`
- `git untrack|log|diff`
- `todo toggle-important|edit|swap|promote|demote|list`
- `split vsplit|hsplit|close|focus|open|direction|linked`

## txv-widgets Changes

- New: `TabPanel`, `SplitPanel`, `TiledWorkspace`
- New: `TabBar` with Single/Static/Lru modes, powerline caps, badges
- New: Transparent cell compositing in blit
- New: View identity (`as_any`/`as_any_mut`) for type-safe downcasting
- Removed: `TabGroup`, `ToolsPanel` (replaced by TabPanel)
- Moved: `SplitDir` now lives in `split_panel` (re-exported from tiled_workspace)
