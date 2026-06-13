---
name: implement-feature
description: Test-first implementation cycle for kairn. Use when implementing a feature or fix. Writes failing test, implements, verifies with lint per file, builds, runs tests, commits.
---

# Implement Feature Skill

## When to Use
When implementing a new feature or fixing a bug in kairn.

## Procedure

1. **Estimate** — Set LOE on the task. If LOE > 3, split into subtasks first.

2. **Test First** — Write a scenario test in `tests/` that demonstrates the desired behavior. The test MUST fail initially.
   - Use `TestHarness` from `tests/helpers.rs`
   - Assert on visible screen output (`screen_text()`, `content_contains()`, `row()`)

3. **Implement** — Write minimal code to make the test pass.
   - After modifying EACH file: run `check_file` on it, fix violations immediately
   - Follow CONVENTIONS.md (max 240 lines, no unwrap/expect, no pub fields)

4. **Verify** — Run the full cycle:
   - `cargo fmt`
   - `cargo build`
   - `cargo clippy -- -D warnings`
   - `cargo test`

5. **Commit** — Stage, commit with descriptive message, push.
   - Commit per subtask (small, focused changes)
   - Pre-commit hook must pass (never skip with --no-verify)

## Two-Repo Rule
- txv (framework) at `../txv` — only change if task explicitly requires framework changes
- kairn (app) at `.` — main workspace
- Both must build after changes

## Context Budget
- If context is running low: commit current progress, mark partial, stop
- Prefer small focused changes over large refactors
