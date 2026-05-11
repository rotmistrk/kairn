# Task: Backscroll Buffer for PTY Terminals

## Objective
Add a configurable scrollback ring buffer (default: 2000 lines) to TermBuf.
PgUp/PgDn in terminal views scroll through scrollback history.

## Context
- Design doc: `doc/f4-design/v-014-session-mcp-todo.md` (Feature 1)
- TermBuf is in `txv-render/src/termbuf/`
- PtyTerminal view is in `txv-widgets/src/pty_terminal.rs`
- Configuration loaded in `src/config.rs` (`.kairnrc`)

## Requirements

1. **Ring buffer in TermBuf** (`txv-render/src/termbuf/scrollback.rs`):
   - `VecDeque<Vec<TCell>>` capped at `scrollback_limit`
   - When `scroll_up()` pushes a line off the top of the visible grid, append it to scrollback
   - Expose `scrollback_len()`, `scrollback_line(offset)` methods
   - `scrollback_limit` passed to `TermBuf::new(cols, rows, scrollback_limit)`

2. **PtyTerminal scrollback navigation**:
   - Track `scroll_offset: usize` (0 = live, >0 = scrolled back)
   - PgUp: increase scroll_offset (up to scrollback_len)
   - PgDn: decrease scroll_offset (min 0)
   - Any keypress other than PgUp/PgDn resets scroll_offset to 0
   - When scroll_offset > 0, draw from scrollback + visible grid combined

3. **Configuration**:
   - Add `scrollback_lines: u16` to settings (default 2000)
   - Pass to PtyTerminal/TermBuf on construction

4. **Public API for MCP** (later consumed by MCP server):
   - `PtyTerminal::get_content(max_lines: usize) -> Vec<String>` — returns last N lines from scrollback + visible

## Constraints
- 240 code lines per file max
- No unwrap/expect/panic in runtime code
- All existing tests must pass
- New tests for scrollback ring buffer behavior

## Files to Create/Modify
- CREATE: `txv-render/src/termbuf/scrollback.rs`
- MODIFY: `txv-render/src/termbuf/mod.rs` (integrate scrollback)
- MODIFY: `txv-widgets/src/pty_terminal.rs` (scroll navigation + draw)
- MODIFY: `src/config.rs` (scrollback_lines setting)
- MODIFY: `src/views/terminal.rs` or `src/build_desktop.rs` (pass config)
