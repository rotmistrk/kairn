# v-005 — Parking Lot (non-txv considerations for later)

Items from the design conversation that are not yet fully specified.
These are parked here so we can focus on txv without losing them.

## Autosave

- File autosave on a timer (e.g., every 30s if modified)
- Configurable: `set auto-save on`, `set auto-save-interval 30`
- Save to the actual file (atomic temp+rename), not a swap file
- Disabled by default — opt-in via config

## Single binary with embedded defaults

- All default configs compiled into the binary via `include_str!`
- `kairn --init-config` writes `~/.kairnrc.tcl` from the embedded template
- If no config file exists, embedded defaults are used silently
- Config template includes commented-out examples of all options

## Installation and dependency setup

- kairn itself: single binary, `cargo install` or download
- LSP servers: not bundled — user installs separately
- Possible: `kairn --setup` command that detects missing LSP servers
  and prints install instructions (or runs them with confirmation)
- Kiro: optional dependency — kairn works without it, Kiro tab just
  shows "kiro-cli not found" message
- Decoupled: kairn does not depend on kiro for any core functionality

## Windows/tabs mode details

- Not fully designed yet — how does window splitting work?
- Horizontal split? Vertical split? Both?
- How do windows interact with the shared bottom panel?
- Deferred to Phase 3 design.

## Dot-repeat (vim `.` command)

- Deferred in feature/mini-ide ("complex, needs careful design")
- Needs a "last edit action" recorder at the command level
- Important for vim users — should be in Phase 1 or Phase 4

## Mouse support

- Deprioritized per user preference
- If added later: click to position cursor, click to focus panel,
  scroll wheel for scrolling. No drag selection initially.

## Multiple cursors

- Deprioritized per user preference
- The command dispatch model supports it (commands operate on a cursor,
  multiple cursors = run command N times) but UI design is non-trivial

## Rusticlish: shell built on rusticle (Deliverable 5)

A shell where structured data (TclValues) flows through pipes instead of
strings. Readline, completion, job control, rich output via rusticle-tk.

- Default shell for kairn's terminal panel
- Speaks the same language as kairn's config and scripting
- Structured pipes: `files "." | lfilter {f { $f("size") > 1000 }} | table`
- LOE: ~8–12 weeks (4–5 without job control)
- Build after kairn is usable — the infrastructure is already done
