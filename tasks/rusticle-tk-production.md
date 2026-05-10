# Task: Make rusticle-tk Production-Ready

## Context

rusticle-tk is a TUI application framework where rusticle (Tcl) scripts define
the UI, rendered via txv-widgets. It's currently a prototype (2317 lines, not in
workspace, references stale `txv` crate).

## Goals

1. Add to workspace, fix dependencies (txv-core + txv-render + txv-widgets, not `txv`)
2. Apply all project standards (pre-commit hook must pass)
3. Include StatusBar with items (clock, mode, position, command input)
4. Proper file structure (240 code line max per file)
5. Tests (unit + scenario)

## Current State

- `src/tk_bridge.rs` (1220 lines!) — the main bridge between rusticle and txv
- `src/event_mgr.rs` (381 lines) — event handling
- `src/layout_mgr.rs` (333 lines) — layout computation
- `src/widget_mgr.rs` (278 lines) — widget management
- `src/main.rs` (97 lines) — entry point
- Dependencies reference `txv` (doesn't exist anymore — split into txv-core/render/widgets)

## Steps

### 1. Add to workspace and fix deps

- Add `"rusticle-tk"` to workspace members in root `Cargo.toml`
- Update `rusticle-tk/Cargo.toml` deps: replace `txv` with `txv-core`, `txv-render`, `txv-widgets`
- Fix all import paths (`txv::` → `txv_core::`, etc.)
- Build successfully

### 2. Apply code standards

- Run `cargo fmt`
- Run `cargo clippy -- -D warnings` and fix all issues
- Split files over 240 code lines (tk_bridge.rs needs 5+ splits, event_mgr 2, layout_mgr 2)
- NO unwrap/expect/panic in runtime code
- Follow the conceptual split SOP (see `.kiro/steering/steering.md`)

### 3. Add StatusBar

- Use `build_status_bar` pattern from kairn or create rusticle-tk's own
- Include: clock, mode indicator, key hints, M-x command input
- StatusBar should be configurable from the rusticle script

### 4. Add Program structure

- Use `txv_core::program::Program` as the run loop (same as kairn)
- Proper enter/leave terminal handling
- Panic handler (same pattern as kairn main.rs)

### 5. Tests

- Unit tests for tk_bridge commands (widget creation, layout, events)
- Scenario tests using MockBackend (same pattern as kairn tests/helpers.rs)
- All tests must be deterministic and independent

### 6. Documentation

- Update README with usage examples
- Document the rusticle-tk Tcl API (widget commands, layout commands, event bindings)

## Key References

- `.kiro/steering/steering.md` — all project SOPs
- `src/status.rs` — how kairn builds its status bar
- `tests/helpers.rs` — TestHarness pattern for scenario tests
- `txv-core/src/view.rs` — View trait (delegate macros, can_close, etc.)
- `doc/f4-design/v-013-txv-architecture.md` — TXV design

## Constraints

- Pre-commit hook MUST pass (fmt, clippy -D warnings, 240 lines, tests)
- Every test must be deterministic and run independently
- No `unwrap()`/`expect()`/`panic!()` in runtime code
- Conceptual splits only (never collapse code to fit line limit)

## Running

```bash
cd /Users/rotmistr/workplace/kairn  # or /home/rotmistr/Workplace/kairn/f4 on remote
cargo build -p rusticle-tk
cargo test -p rusticle-tk
```
