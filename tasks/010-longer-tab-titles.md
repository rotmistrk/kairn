# Longer Tab Titles

<!-- TODO: mark done → todo tree [2][10] -->

## Problem

Tab titles are currently truncated too aggressively. Shell and Kiro tabs
often have useful context in their title (OSC title, agent name, command)
that gets cut off.

## Solution

Allow tab titles up to 60 characters. Dynamically shrink based on available
panel width so that all tabs + badges fit.

## Algorithm

```
chrome_overhead = panel_border + count_badge + padding
badge_width = per-tab badge glyph (2 chars)
separator_width = powerline glyph (1 char)

available = panel_width - chrome_overhead
per_tab = available / num_visible_tabs - separator_width - badge_width
max_title_len = clamp(per_tab, 8, 60)
```

When a title exceeds `max_title_len`, truncate with `…`:
```
"Kiro:0 reviewing handler.rs for err…"
```

## Behavior

- Wide panels: titles can use up to 60 chars
- Narrow panels: titles shrink proportionally
- Minimum: 8 chars (enough for "Kiro:0…")
- Focused tab gets priority (slightly more space if needed)

## Files to Modify

- `txv-widgets/src/tab_group_view.rs` — title truncation logic in draw_chrome

## Constraints

- 240 code line max per file
- Must account for badge width (from pty-activity-badges task)
- Must handle wide characters in titles correctly
