# PTY Activity Badges

<!-- TODO: mark done → todo tree [2][8] (auto-close) and [2][9] (badges) -->

## Overview

Show per-tab activity indicators in the tab chrome and an aggregate badge
on the tools panel, so you can see at a glance which terminals are busy,
idle, or exited.

## Per-Tab Badges

Rendered after the tab title, before the Powerline separator:

| State | Glyph | Color | Condition |
|-------|-------|-------|-----------|
| Busy | `⟳` or `◉` | green/cyan | Output received in last N seconds |
| Idle/waiting | `●` | yellow | No output for N seconds (prompt returned) |
| Exited OK | (auto-closed) | — | Exit code 0, tab closes |
| Exited error | `✗` | red | Non-zero exit, awaiting close |

### Detection

Track `last_output_timestamp` per PTY tab. On each tick:
- If `now - last_output > idle_timeout` → idle
- Otherwise → busy
- On child exit with code != 0 → error state

### Configuration

```tcl
set terminal.idle-timeout 3          ;# seconds before idle badge
set terminal.auto-close-on-exit true ;# close tab on exit code 0
```

## Aggregate Badge on Tools Panel

In the panel chrome count area, show activity summary:

```
 Tools ❨4❩ 2⟳ 1●
```

Or simpler: color the `❨N❩` count badge based on aggregate state:
- Green if any tab is active/busy
- Yellow if all are idle
- Red if any exited with error

## Auto-Close on Exit

- Exit code 0 → auto-close tab (configurable via `terminal.auto-close-on-exit`)
- Exit code != 0 → show `✗` badge, tab stays open
- Close with `:close` / `M-x close` / any configured hotkey
- Tcl hook: `hook add pty-exit { view message "PTY exited: $exit_code" }`

## Implementation

1. Add `last_output_ts: Instant` and `exit_status: Option<i32>` to PTY tab state
2. Update `last_output_ts` whenever PtyTerminal produces output
3. In chrome draw: check state, render appropriate glyph + color
4. On child exit: if code == 0 and auto-close enabled → emit CM_TAB_CLOSE
5. On child exit: if code != 0 → set exit_status, mark badge as error
6. Aggregate: iterate tools panel tabs, count busy/idle/error

## Tab Title Length

Allow tab titles up to 60 characters. When panel width is tight, truncate
titles to fit all badges. Algorithm:

```
available = panel_width - chrome_overhead - badge_width
max_title = min(60, available / num_visible_tabs - separator_width)
```

Truncate with `…` when title exceeds max_title.

## Files to Modify

- `txv-widgets/src/tab_group_view.rs` — badge rendering in chrome
- `txv-widgets/src/pty_terminal.rs` — expose last_output_ts, exit_status
- `src/views/terminal.rs` — wire exit detection
- `src/handler.rs` — handle auto-close on exit
- `src/config.rs` — idle-timeout and auto-close settings

## Constraints

- 240 code line max per file
- No unwrap/expect/panic
- Badge rendering must not flicker (only redraw on state change)
- Must work with both wide and tall layouts
