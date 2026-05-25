# Kairn Tcl Scripting Reference

Kairn embeds a Tcl interpreter for configuration, automation, and extensibility.
Any M-x command that isn't a built-in is evaluated as Tcl.

## Configuration

Scripts are loaded in order:
1. `~/.kairn/config.tcl` — global user config
2. `.kairn/init.tcl` — project-specific config

Settings use `set variable value` syntax. See `doc/example-init.tcl` for all options.

## Namespaces

### editor

| Command | Description |
|---------|-------------|
| `editor open <path> ?-line N? ?-col N?` | Open file |
| `editor save` | Save current file |
| `editor save-all` | Save all modified files |
| `editor close` | Close current tab |
| `editor undo` | Undo last edit |
| `editor redo` | Redo last undone edit |
| `editor goto <line> ?<col>?` | Jump to position (1-indexed) |
| `editor insert <text>` | Insert text at cursor |
| `editor search <pattern>` | Highlight search matches |
| `editor clear-highlight` | Clear search highlighting |
| `editor get-selection` | Returns selected text |
| `editor replace-selection <text>` | Replace selection |
| `editor get-line ?<n>?` | Get line text (default: cursor line) |
| `editor delete-line ?<n>?` | Delete line |
| `editor replace-word <text>` | Replace word under cursor |
| `editor diff-revert` | Revert diff hunk at cursor |
| `editor current-file` | Returns current file path |
| `editor current-line` | Returns cursor line (1-indexed) |
| `editor current-col` | Returns cursor column (1-indexed) |
| `editor modified?` | Returns 1 if buffer is modified |
| `editor filetype` | Returns file extension |

### view

| Command | Description |
|---------|-------------|
| `view focus left\|center\|right` | Focus a panel |
| `view theme dark\|light\|auto\|toggle` | Set color mode |
| `view zoom` | Toggle maximize current panel |
| `view toggle-tree` | Show/hide file tree panel |
| `view toggle-tools` | Show/hide tools panel |
| `view layout` | Cycle layout mode (auto/wide/tall) |
| `view message <level> <origin> <text>` | Show message (level: info/warn/error) |
| `view status <text>` | Flash text in status bar |

### build

| Command | Description |
|---------|-------------|
| `build run ?<cmd>?` | Run build command |
| `build test ?<cmd>?` | Run test command |
| `build test-file` | Test current file |
| `build test-at-cursor` | Test function at cursor |
| `build next-error` | Jump to next diagnostic |
| `build prev-error` | Jump to previous diagnostic |

### lsp

| Command | Description |
|---------|-------------|
| `lsp hover` | Show hover information |
| `lsp definition` | Go to definition |
| `lsp references` | Find all references |
| `lsp rename <new-name>` | Rename symbol |
| `lsp format` | Format document |
| `lsp start ?<pattern>?` | Start LSP server (pattern matches server name) |
| `lsp restart ?<pattern>?` | Restart LSP server |
| `lsp stop ?<pattern>?` | Stop LSP server |
| `lsp timeout <pattern> ?<secs>?` | Get/set LSP timeout |
| `lsp args <pattern> <command>` | Override LSP server command |

### git

| Command | Description |
|---------|-------------|
| `git stage <file>` | Stage a file |
| `git unstage <file>` | Unstage a file |
| `git untrack <file>` | Untrack a file |
| `git commit <message>` | Commit with message |
| `git blame` | Show blame annotations |
| `git noblame` | Hide blame annotations |
| `git log` | Show git log |
| `git diff` | Show diff for current file |

### todo

Paths are dot-separated indices (e.g., `0.2.1` = first item → third child → second grandchild).

| Command | Description |
|---------|-------------|
| `todo add <text> ?-parent <path>?` | Add item (sibling after path, or top-level) |
| `todo remove <path>` | Remove item |
| `todo complete <path>` | Toggle completion |
| `todo toggle-important <path>` | Toggle important flag |
| `todo edit <path> <text>` | Rename item |
| `todo swap <path> up\|down` | Reorder within siblings |
| `todo promote <path>` | Decrease nesting (move up a level) |
| `todo demote <path>` | Increase nesting (make child of previous sibling) |
| `todo list` | Reserved (tree panel shows todos) |

### split

| Command | Description |
|---------|-------------|
| `split vsplit ?<file>?` | Create vertical split |
| `split hsplit ?<file>?` | Create horizontal split |
| `split close` | Close split (keep focused pane) |
| `split focus` | Cycle focus between panes |
| `split open <path>` | Open file in other pane |
| `split linked ?<bool>?` | Get/set linked scroll |
| `split direction` | Returns current split direction |

### grep

| Command | Description |
|---------|-------------|
| `grep <pattern>` | Search project files, open results tab |

### keymap

| Command | Description |
|---------|-------------|
| `keymap bind <key> <command>` | Bind key to Tcl command |
| `keymap unbind <key>` | Remove key binding |

Key format: `ctrl+x`, `alt+x`, `F1`–`F12`, `ctrl+shift+x`, etc.

### hook

| Command | Description |
|---------|-------------|
| `hook add <event> ?-filter <pat>? <script>` | Register hook |
| `hook remove <id>` | Remove hook by ID |
| `hook list ?<event>?` | List registered hooks |

Events: `char-inserted`, `word-completed`, `idle`, `file-opened`, `file-saved`.

### system

| Command | Description |
|---------|-------------|
| `system exec <cmd>` | Run shell command, return stdout |
| `system env <var>` | Get environment variable |
| `system set-env <var> <val>` | Set environment variable |
| `system root-dir` | Returns project root directory |
| `system home-dir` | Returns user home directory |
| `system platform` | Returns OS name |
| `system clipboard-get` | Read system clipboard |
| `system clipboard-set <text>` | Write to system clipboard |

## Build/Test/Run Overrides

Define Tcl procs to replace the auto-detected build commands. If the proc returns
a non-empty string, it replaces the default. Return `""` to fall back to auto-detection.

| Proc name | Overrides |
|-----------|-----------|
| `build-command` | `:build` / M-x build |
| `test-command` | `:test` / M-x test |
| `run-command` | `:run` / M-x run |

```tcl
# Global override in ~/.kairn/config.tcl
proc build-command {} { return "make -j8" }

# Project-specific in .kairn/init.tcl
proc test-command {} {
    set file [editor current-file]
    if {[string match "*.go" $file]} {
        return "go test ./..."
    }
    return ""  ;# fall back to auto-detect
}

# Context-aware: test the current package
proc test-command {} {
    set file [editor current-file]
    set dir [file dirname $file]
    return "cargo test -p [file tail $dir]"
}
```

Project `.kairn/init.tcl` loads after global config, so project procs override global ones.

## Hooks

Hooks fire on editor events. Use `-filter` to match specific triggers:

```tcl
# Auto-close brackets
hook add char-inserted -filter "(" { editor insert ")" }
hook add char-inserted -filter "{" { editor insert "}" }

# Expand abbreviations
hook add word-completed -filter "todo" {
    editor replace-word "// TODO(user): "
}

# Format on idle
hook add idle { lsp format }
```

## Key Bindings

```tcl
# Bind Ctrl+Q to wrap selection in quotes
keymap bind ctrl+q {
    set sel [editor get-selection]
    editor replace-selection "\"$sel\""
}

# Bind Alt+G to grep word under cursor
keymap bind alt+g {
    set word [editor get-selection]
    if {$word eq ""} { set word [editor current-word] }
    grep $word
}
```

## Project Root Override

Define a `project-root` proc to override automatic root detection:

```tcl
proc project-root {path} {
    if {[string match "*/monorepo/*" $path]} {
        return "/home/user/monorepo"
    }
    return ""
}
```

## MCP Integration

The `eval_tcl` MCP tool allows AI agents to execute any Tcl command:

```json
{"name": "eval_tcl", "arguments": {"script": "editor goto 42"}}
```

This makes the entire Tcl API available to AI without needing dedicated MCP tools for every operation.

## LSP Preamble

The Tcl LSP (`rusticle-lsp`) needs to know which commands exist to avoid false
"unknown command" diagnostics. Two mechanisms provide this:

### Project Prelude (--prelude)

Kairn passes `--prelude .kairn/prelude.tcl` when starting the LSP. This file
contains stub `proc` declarations for all kairn bridge commands:

```tcl
# .kairn/prelude.tcl
proc editor {subcmd args} {}
proc view {subcmd args} {}
proc build {subcmd args} {}
# ... etc
```

The LSP evaluates these stubs at startup so it recognizes `editor`, `view`, etc.
as valid commands. The stubs have no implementation — they only suppress diagnostics
and enable completion.

### Shebang Discovery (--lsp-preamble)

For standalone Tcl scripts with a shebang (e.g. `#!/usr/bin/env rusticle-tk`),
the LSP automatically discovers commands by running:

```
<interpreter> --lsp-preamble
```

The interpreter outputs its command stubs to stdout (one `proc name {args} {}`
per line). Results are cached per interpreter for the LSP session lifetime.
If the command fails or times out (2s), an empty preamble is cached.

To support this in your own Tcl-based tool, add a `--lsp-preamble` flag that
prints proc stubs for all custom commands your tool registers.
