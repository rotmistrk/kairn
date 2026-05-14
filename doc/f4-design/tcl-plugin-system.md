# Kairn Plugin System — Tcl Bridge Design

## Vision

Every user-facing action in kairn is a **command**. Commands are Tcl procs.
The editor is a Tcl-scriptable environment where:
- All built-in features are exposed as Tcl commands
- Users extend behavior by writing Tcl scripts
- Keybindings, hooks, views, and parsers are all configurable via Tcl
- M-x evaluates Tcl expressions (with completion from the command registry)
- Plugins are directories containing `init.tcl` loaded at startup

## Architecture

```
┌─────────────────────────────────────────────────┐
│  Kairn TUI (txv views, event loop)              │
├─────────────────────────────────────────────────┤
│  Command Dispatch Layer                         │
│  ┌───────────┐  ┌──────────┐  ┌─────────────┐  │
│  │ KeyMap    │  │ M-x      │  │ Hooks       │  │
│  │ key→cmd   │  │ tcl eval │  │ event→procs │  │
│  └─────┬─────┘  └────┬─────┘  └──────┬──────┘  │
│        │              │               │         │
│        ▼              ▼               ▼         │
│  ┌─────────────────────────────────────────┐    │
│  │  Command Registry                       │    │
│  │  name → CommandDef { proc, desc, cat }  │    │
│  └─────────────────┬───────────────────────┘    │
│                    │                            │
├────────────────────┼────────────────────────────┤
│  Tcl Bridge        │                            │
│  ┌─────────────────▼───────────────────────┐    │
│  │  Rusticle Interpreter                   │    │
│  │  - Built-in procs (Rust → Tcl bridge)   │    │
│  │  - User procs (from config/plugins)     │    │
│  │  - Variable namespace (editor state)    │    │
│  └─────────────────────────────────────────┘    │
├─────────────────────────────────────────────────┤
│  Core APIs (exposed to Tcl)                     │
│  - editor: open, save, goto, insert, ...        │
│  - buffer: get-line, set-line, selection, ...    │
│  - view: create, focus, close, ...              │
│  - build: run, parse-errors, ...                │
│  - lsp: hover, definition, references, ...      │
│  - system: exec, env, path, ...                 │
└─────────────────────────────────────────────────┘
```

## Command Registry

Every action is a named command with metadata:

```rust
pub struct CommandDef {
    pub name: String,           // "file-save", "build", "grep"
    pub description: String,    // "Save current file"
    pub category: Category,     // File, Edit, View, Build, LSP, Plugin
    pub binding: Option<String>,// "Ctrl-S" (from keymap)
    pub proc_name: String,      // Tcl proc to call
}

pub enum Category {
    File, Edit, View, Navigate, Build, Test, LSP, Git, Plugin,
}
```

Commands are registered at startup:
1. Built-in commands registered by Rust code
2. Plugin commands registered by `init.tcl` scripts
3. User commands from `~/.kairn/config.tcl`

## Tcl Bridge API

### Core Namespace: `editor`

```tcl
editor open <path> ?-line N? ?-col N?    ;# open file in main panel
editor save                               ;# save current file
editor save-as <path>                     ;# save to new path
editor close                              ;# close current tab
editor goto <line> ?<col>?               ;# jump to position
editor insert <text>                      ;# insert at cursor
editor delete-line                        ;# delete current line
editor selection                          ;# return selected text
editor set-selection <start> <end>        ;# set selection range
editor current-file                       ;# return current file path
editor current-line                       ;# return cursor line (0-indexed)
editor current-col                        ;# return cursor column
editor line-count                         ;# number of lines in buffer
editor get-line <n>                       ;# return line content
editor word-at-cursor                     ;# return word under cursor
editor modified?                          ;# return 1 if buffer dirty
```

### Core Namespace: `buffer`

```tcl
buffer get <start-line> <end-line>        ;# return lines as list
buffer replace <start> <end> <text>       ;# replace range
buffer indent <start> <end>               ;# indent lines
buffer comment <start> <end>              ;# toggle line comments
```

### Core Namespace: `view`

```tcl
view focus <slot>                         ;# focus Left/Center/Right/Bottom
view open-panel <title> <content>         ;# open text in tool panel
view message <level> <origin> <text>      ;# show in status + messages
view status <text>                        ;# flash in status bar
view prompt <message> ?-default val?      ;# input dialog, returns value
view confirm <message>                    ;# yes/no dialog, returns 0/1
view results <title> <entries>            ;# open results view
                                          ;# entries = list of {path line col text}
```

### Core Namespace: `build`

```tcl
build run <command>                       ;# run async, show in results
build parse-line <line>                   ;# parse error line → {path line col text} or ""
build detect                              ;# return detected build command
build set-command <cmd>                   ;# override build command
build set-test-command <cmd>              ;# override test command
build set-parser <proc>                   ;# set custom error parser proc
```

### Core Namespace: `lsp`

```tcl
lsp hover                                 ;# show hover at cursor
lsp definition                            ;# goto definition
lsp references                            ;# find references
lsp rename <new-name>                     ;# rename symbol
lsp completions                           ;# trigger completion
lsp diagnostics ?<file>?                  ;# return diagnostics list
```

### Core Namespace: `system`

```tcl
system exec <command>                     ;# run synchronously, return output
system exec-async <command> ?-on-line proc? ?-on-done proc?
                                          ;# run async with callbacks
system env <var>                          ;# get env variable
system root-dir                           ;# workspace root
system platform                           ;# "macos", "linux"
system clipboard-get                      ;# read clipboard
system clipboard-set <text>               ;# write clipboard
```

### Core Namespace: `keymap`

```tcl
keymap bind <key> <command>               ;# bind key to command
keymap unbind <key>                       ;# remove binding
keymap mode <name>                        ;# switch to keymap mode
keymap define-mode <name> <parent>        ;# create new mode inheriting from parent
```

### Core Namespace: `hook`

```tcl
hook add <event> <proc>                   ;# register hook
hook remove <event> <proc>                ;# unregister hook

# Available events:
#   file-open {path}
#   file-save {path}
#   file-close {path}
#   buffer-modified {path}
#   cursor-moved {line col}
#   mode-changed {mode}
#   build-done {exit-code}
#   build-error {path line col text}
#   tab-switched {title}
#   startup {}
#   shutdown {}
#   todo-item-completed {id label}
#   todo-item-created {id label}
```

### Core Namespace: `plugin`

```tcl
plugin register <name> {
    version <ver>
    description <desc>
    commands { cmd1 cmd2 ... }
    hooks { event1 event2 ... }
}
plugin list                               ;# return registered plugins
plugin enabled? <name>                    ;# check if plugin active
```

## Keybindings

Keymaps are Tcl scripts that call `keymap bind`:

```tcl
# vim.tcl (default)
keymap define-mode normal {}
keymap define-mode insert {}
keymap define-mode visual {}

keymap mode normal
keymap bind "i"         "mode-insert"
keymap bind "v"         "mode-visual"
keymap bind "dd"        "delete-line"
keymap bind "gd"        "lsp definition"
keymap bind "gr"        "lsp references"
keymap bind "K"         "lsp hover"
keymap bind ":"         "command-line"
keymap bind "/"         "search-forward"
keymap bind "Ctrl-N"    "completion-next"

keymap mode insert
keymap bind "Escape"    "mode-normal"
keymap bind "Ctrl-N"    "completion-next"
keymap bind "Ctrl-P"    "completion-prev"
```

```tcl
# vscode.tcl
keymap define-mode default {}
keymap mode default
keymap bind "Ctrl-Shift-P"  "command-palette"
keymap bind "Ctrl-S"        "file-save"
keymap bind "Ctrl-P"        "file-open-fuzzy"
keymap bind "Ctrl-Shift-F"  "grep-prompt"
keymap bind "F5"            "build"
keymap bind "Ctrl-`"        "toggle-terminal"
keymap bind "Ctrl-B"        "toggle-sidebar"
keymap bind "F12"           "lsp definition"
keymap bind "Shift-F12"     "lsp references"
keymap bind "Ctrl-."        "lsp code-action"
keymap bind "F2"            "lsp rename"
```

## M-x Command Line

When user presses M-x (or `:` in vim mode):
1. Open FuzzySelect populated from command registry
2. User types → fuzzy filter on command names + descriptions
3. Each entry shows: `command-name    description    [binding]`
4. Enter → evaluate as Tcl: `interp.eval(selected.proc_name)`
5. If command needs arguments, it opens a prompt (via `view prompt`)

For raw Tcl evaluation: prefix with `!`:
```
M-x: !editor goto 42
M-x: !set x [editor current-line]; puts "Line: $x"
```

## Status Bar Items (Plugin-defined)

Plugins can add status bar items via Tcl:

```tcl
# Git branch indicator (plugin)
statusbar add "git-branch" {
    gravity right
    update-on { file-open file-save }
    proc { return [exec git branch --show-current] }
}

# Word count (plugin)
statusbar add "word-count" {
    gravity right
    update-on { buffer-modified cursor-moved }
    proc {
        set text [editor selection]
        if {$text eq ""} { set text [buffer get 0 [editor line-count]] }
        return "[llength [split $text]] words"
    }
}
```

Implementation: `StatusBarItem` that holds a Tcl proc name, calls it on tick/event,
displays the return value as its label.

## Hooks — View-Specific

### Editor hooks
```tcl
hook add file-open {path} {
    # Auto-detect indent
    if {[string match "*.py" $path]} {
        editor set-option indent-width 4
        editor set-option use-tabs 0
    }
}

hook add file-save {path} {
    # Auto-format on save
    if {[string match "*.rs" $path]} {
        system exec "rustfmt $path"
        editor reload
    }
}
```

### Todo hooks
```tcl
hook add todo-item-completed {id label} {
    # Log completion time
    set ts [clock format [clock seconds]]
    todo set-note $id "Completed: $ts"
}

hook add todo-item-created {id label} {
    # Auto-assign due date
    if {[string match "*urgent*" $label]} {
        todo set-priority $id high
    }
}
```

### Build hooks
```tcl
hook add build-done {exit_code} {
    if {$exit_code == 0} {
        view status "✓ Build succeeded"
        # Auto-run tests after successful build
        build run [build detect-test]
    } else {
        view status "✗ Build failed"
    }
}

hook add build-error {path line col text} {
    # Custom error annotation
    editor annotate $path $line "⚠ $text"
}
```

## Plugin Examples

### Custom Build Parser Plugin

```tcl
# ~/.kairn/plugins/pytest-parser/init.tcl

plugin register "pytest-parser" {
    version "1.0"
    description "Parse pytest output for error navigation"
}

# Custom parser for pytest output:
# FAILED tests/test_foo.py::test_bar - AssertionError: ...
proc pytest-parse-line {line} {
    if {[regexp {^FAILED (.+)::(.+) - (.+)$} $line _ file test msg]} {
        # Find line number by searching for test function
        set content [system exec "grep -n 'def $test' $file"]
        if {[regexp {^(\d+):} $content _ lineno]} {
            return [list $file $lineno 0 "$test: $msg"]
        }
    }
    return ""
}

# Register as build parser for python projects
hook add startup {} {
    if {[file exists "pytest.ini"] || [file exists "setup.py"]} {
        build set-command "pytest --tb=line 2>&1"
        build set-parser pytest-parse-line
    }
}
```

### Auto-save Plugin

```tcl
# ~/.kairn/plugins/autosave/init.tcl

plugin register "autosave" {
    version "1.0"
    description "Auto-save files after 5 seconds of inactivity"
}

set autosave_delay 5000  ;# ms

hook add buffer-modified {path} {
    after cancel autosave-fire
    after $autosave_delay autosave-fire
}

proc autosave-fire {} {
    if {[editor modified?]} {
        editor save
        view status "Auto-saved"
    }
}
```

### Snippet Plugin

```tcl
# ~/.kairn/plugins/snippets/init.tcl

plugin register "snippets" {
    version "1.0"
    description "Text snippets with tab expansion"
    commands { snippet-expand snippet-add }
}

# Snippets stored as Tcl dict
set snippets [dict create]

proc snippet-add {trigger body} {
    dict set ::snippets $trigger $body
}

proc snippet-expand {} {
    set word [editor word-at-cursor]
    if {[dict exists $::snippets $word]} {
        set body [dict get $::snippets $word]
        # Delete trigger word
        editor delete-word-back
        # Insert snippet (with cursor placeholder handling)
        editor insert $body
    }
}

# Register default snippets per filetype
hook add file-open {path} {
    if {[string match "*.rs" $path]} {
        snippet-add "fn" "fn ${1:name}(${2:args}) -> ${3:Type} {\n    ${0}\n}"
        snippet-add "impl" "impl ${1:Type} {\n    ${0}\n}"
    }
}

keymap bind "Tab" "snippet-expand"
```

## Configuration Loading Order

1. Built-in commands registered (Rust)
2. `~/.kairn/keymap.tcl` — keybindings (default: vim.tcl)
3. `~/.kairn/config.tcl` — user preferences
4. `~/.kairn/plugins/*/init.tcl` — plugins (alphabetical)
5. `.kairn/init.tcl` — project-specific config (overrides)

## Implementation Plan

### Phase 1: Foundation
- Embed rusticle interpreter in kairn (`src/scripting/mod.rs`)
- Register core `editor` and `system` commands as Tcl procs
- M-x evaluates Tcl (`:` in vim mode still works as before)
- Load `~/.kairn/config.tcl` at startup

### Phase 2: Command Registry + Palette
- `CommandRegistry` struct with all commands
- M-x opens FuzzySelect from registry
- Commands have name, description, category, binding

### Phase 3: Keymaps
- `keymap bind/unbind/mode` Tcl commands
- Load keymap file at startup
- Ship vim.tcl, vscode.tcl, emacs.tcl

### Phase 4: Hooks
- `hook add/remove` Tcl commands
- Fire hooks from Rust at appropriate points
- Async hooks (don't block UI)

### Phase 5: Plugins
- Plugin loader scans `~/.kairn/plugins/`
- `plugin register` Tcl command
- Plugin enable/disable via config

### Phase 6: View API
- `view` namespace for creating panels, results, prompts
- Status bar items from Tcl
- Custom views (Tcl-driven draw? Or just data views?)

## Constraints

- Tcl evaluation MUST NOT block the UI (use async for long operations)
- Errors in Tcl scripts MUST show in messages (never silent)
- Plugins cannot crash kairn (catch all Tcl errors)
- Built-in commands remain fast (Rust) — Tcl is glue, not hot path
- No `exec` of external tools in core — plugins may use `system exec` at their own risk
