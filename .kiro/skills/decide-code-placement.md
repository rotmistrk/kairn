---
name: decide-code-placement
description: Decide whether a change belongs in txv (framework) or kairn (app). Use before implementing any feature that could be reusable.
---

# Decide Code Placement

## When to Use
Before implementing any feature, ask: where does this code belong?

## Decision Criteria

### Goes in txv (framework) if:
- It's an **architectural pattern** (encapsulation, polymorphism, event pipeline)
- It's a **general UI component** reusable across different TUI applications (tree view, dropdown, input line, diff viewer, tab bar)
- It **doesn't depend** on kairn-specific features, commands, or data structures
- Other apps (duir, rusticle-tk consumers) would benefit from it

### Goes in txv with delegate/hook pattern if:
- It's **general behavior** but needs app-specific customization
- Examples: EditorView (general) + EditorViewDelegate (kairn provides LSP, diagnostics, blame); DropdownMenu (general) + DropdownSource (app provides data)
- The pattern: txv owns the widget + trait, kairn implements the trait

### Goes in kairn (app) if:
- It's **specific to IDE workflow** (LSP commands, git integration, MCP tools, build system)
- It uses kairn-specific state (AppState, BufferRegistry, workspace roots)
- It would not make sense in a non-IDE TUI application

## Splitting a Feature

When a feature spans both:
1. Identify the **reusable core** — put in txv as a widget/trait
2. Identify the **app-specific hooks** — define as a trait in txv
3. Implement the trait in kairn
4. The txv component must compile and be testable without kairn

## Examples

| Feature | txv | kairn |
|---------|-----|-------|
| Diff rendering | DiffView widget, DiffLine model | Git integration, :diff command, flush_diff |
| Gutter signs | `gutter_sign()` / `gutter_sign_right()` delegate methods | Compute git signs, compute diagnostics |
| Command palette | InputLine + DropdownMenu + Completer trait | AppCompleter, command list, M-x handler |
| Scroll with wrap | Viewport scroll accounting for visual rows | (none — pure framework) |
| Session persistence | (none — app-specific) | Save/restore open tabs, cursor positions |
