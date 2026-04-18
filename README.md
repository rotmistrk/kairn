# kairn

A TUI IDE oriented around [Kiro](https://kiro.dev) AI — file tree, syntax-highlighted viewer, incremental search, unified git diff, session management, and multi-tab Kiro/shell sessions.

## Layouts (Ctrl-L to rotate)

```
Layout 1 (Wide):        Layout 2 (Tall-Right):    Layout 3 (Tall-Bottom):
┌────┬──────┬─────┐    ┌────┬──────────────┐     ┌────┬──────────────┐
│Tree│ Main │Kiro/│    │Tree│    Main      │     │Tree│    Main      │
│    │      │Shell│    │    ├──────────────┤     ├────┴──────────────┤
└────┴──────┴─────┘    │    │  Kiro/Shell  │     │    Kiro/Shell     │
                       └────┴──────────────┘     └──────────────────┘
```

## Key Bindings

| Key | Action |
|---|---|
| Ctrl-Q | Quit |
| Ctrl-L | Rotate layout |
| Ctrl-B | Toggle file tree |
| Ctrl-Tab | Cycle panel focus |
| Ctrl-K | New Kiro tab |
| Ctrl-S | New shell tab |
| Alt-←/→ | Switch tabs |
| Ctrl-W | Close tab |
| Alt-arrows | Resize panels (±1) |
| Alt-Shift-arrows | Resize panels (±5) |
| Ctrl-O | Pin tab output to main panel |

## Building

```bash
cargo build
cargo run
```

## License

MIT
