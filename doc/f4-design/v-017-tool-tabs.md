# Tool Tab Naming Design

## Tab Title Format

All tool tabs follow the pattern: `{Type}:{user_part}`

- **Type prefix** is fixed, set at creation, cannot be renamed by user
- **User part** is the portion after `:` — can be renamed or auto-set

## Tab Types

| Type | Multiple? | Default user_part | Auto-update | Rename? |
|------|-----------|-------------------|-------------|---------|
| Shell | Yes (0..9) | `n` (first available) | PTY title → `n.{title}` | Yes (user_part only) |
| Kiro | Yes (0..9) | `n` (first available) | PTY title → `n.{title}` | Yes (user_part only) |
| Compile | No (singleton) | (empty or target) | — | No |
| Find | No per pattern | `{pattern}` | — | No |
| Test | TBD | TBD | — | TBD |

## Naming Rules

1. On creation: find first available N (0..9) for the type. Max 10 per type.
2. Default title: `Shell:0`, `Shell:1`, `Kiro:0`, etc.
3. When PTY sets terminal title (OSC 0/2): update to `Shell:0.{title}`
4. User rename (via command): replaces user_part → `Shell:{custom_name}`
5. Type prefix is immutable after creation.

## Commands

| Command | Action |
|---------|--------|
| `shell` | Open new Shell tab (next available N) |
| `kiro` | Open new Kiro tab (next available N) |
| `rename {name}` | Rename current tool tab's user_part |
| `close` | Close current tab (works for any tab) |

## Close Behavior

- `close` command (M-x close, or :q in editor) closes the focused tab
- If the closed tab was the last in a slot, slot becomes empty
- For tool tabs: PTY process is killed on close

## Examples

```
Shell:0           — first shell, no PTY title set
Shell:0.vim       — first shell, PTY title is "vim"
Shell:1           — second shell
Kiro:0            — first kiro session
Kiro:0.planning   — kiro session with title "planning"
Shell:myserver    — user renamed to "myserver"
Compile:          — singleton compile tab
Find:TODO         — find results for "TODO"
```
