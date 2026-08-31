# Kairn Agent SOPs

When working inside kairn, follow these SOPs to leverage the IDE integration effectively.

---

## Task Planning: Use the Todo Tree

When given a multi-step task, structure it in the **kairn todo tree** (left panel) before starting implementation.

**Why:** The todo tree persists across sessions, is visible to the user, and allows collaborative planning.

**How:**
1. Use `get_todo_tree` to see current state
2. Use `add_subtree` to create task hierarchy with:
   - Clear, actionable titles
   - Notes containing context, acceptance criteria, or suggested prompts
   - Logical subtask breakdown
3. Use `update_todo` to mark items complete as you progress

**Example structure:**
```
□ Implement feature X
  □ Research: check existing patterns in codebase
  □ Design: propose approach (add note with options)
  □ Implement: core logic
  □ Test: add unit tests
  □ Verify: run pre-commit
```

---

## Code Visibility: Show, Don't Just Tell

Before proposing code changes, **make the relevant code visible** to the user.

**Why:** The user can see exactly what you're discussing, catch misunderstandings early, and learn the codebase.

**How:**
- Use `open_file` + `highlight_code` to show the relevant section before discussing changes
- Use `split` to show related files side-by-side (test + implementation, old + new)
- After making changes, use the editor's `:diff` or diff MCP tools to show before/after

**Don't:** Describe changes in prose without showing the code first.

---

## Build Feedback: Use Visible Terminals

Run builds and tests in **terminal tabs** rather than silent shell commands when the user should see progress.

**Why:** Build output, test failures, and progress are visible in real-time.

**How:**
- For quick checks: `run_build` with `get_build_errors`
- For interactive/long-running: `send_terminal_input` to a shell tab
- Check results with `get_terminal_content`

---

## Progress Communication: Use the Message Ring

Announce milestones and status via the **message ring** (visible in status bar, reviewable via F6).

**Why:** User sees progress without reading every tool output.

**How:**
- Use `eval_tcl` with `view message info <source> <text>` for status updates
- Examples: "Starting phase 2...", "All tests pass", "Waiting for user input"

---

## Context Awareness: Read Before Writing

Before modifying code, **read the relevant context** to understand:
- Existing patterns and conventions
- Related code that might need updates
- Test files that cover the code

**How:**
- Use `get_tab_content` for open files
- Use `search_project` to find related code
- Use `get_diagnostics` to check for existing issues

---

## Diff Review: Show Changes Before Committing

Before committing, **show the user what changed** for review.

**How:**
1. Use `git_ops` with action `stage` to prepare changes
2. Open changed files and use `:diff` to show modifications
3. Wait for user confirmation before `git_ops` commit

---

## Split Views for Comparison

When comparing implementations, reviewing changes, or showing test + code:

**How:**
- Use `split` with `vsplit` or `hsplit` to create side-by-side views
- Enable `linked` scroll for synchronized navigation
- Use `close` to clean up when done

---

## Error Recovery: Check Messages

When something fails unexpectedly:

**How:**
1. Use `get_messages` to see recent errors/warnings
2. Use `get_diagnostics` for LSP errors in specific files
3. Use `get_build_errors` for compilation issues
