# Rusticle Language Reference

Rusticle is a scripting language inspired by Tcl, reimplemented in Rust
with modern enhancements. It keeps Tcl's core model (everything is a
command, `$` substitution, `[]` command substitution, `{}` quoting) while
fixing known pain points and adding features from modern languages.

## Differences from Tcl

### Changed

| Feature | Standard Tcl | Rusticle | Why |
|---------|-------------|----------|-----|
| Scoping | Isolated proc scopes, `global`/`upvar` to access outer | Lexical scoping: reads walk up scope chain, writes local by default | Eliminates the most common Tcl confusion |
| Namespaces | `namespace eval`, `variable`, verbose | `context` blocks with `::` access | Simpler, cleaner |
| Arrays | `set arr(key) val` (associative arrays) | Replaced by dicts + accessor syntax | Dicts are more powerful and consistent |
| Assignment | `set x 42` only | `set x 42` (classic) or `set x = 42` (modern) | `=` form required for destructuring |
| Destructuring | Not available | `set a, b, c = [list 1 2 3]` | Modern convenience |
| Error handling | `catch script ?var?` | `try {} on error {msg} {} finally {}` | Structured, readable |

### Added (not in Tcl)

| Feature | Syntax | Example |
|---------|--------|---------|
| Structured dict literal | `%{ key: value }` | `set d %{ name: "kairn", ver: 1 }` |
| Structured list literal | `%[ item, item ]` | `set l %[ "a", "b", "c" ]` |
| Accessor syntax | `$var(index)`, `$var("key")` | `$mylist(0)`, `$mydict("name")` |
| Properties | `$var.len`, `$var.keys` | `$mylist.len`, `$mydict.keys` |
| Pipe operator | `a \| b` | `$x \| string trim \| string toupper` |
| Optional chaining | `$var("key")?` | `$config("missing")? ?? "default"` |
| Null coalescing | `expr1 ?? expr2` | `$val ?? "fallback"` |
| Range | `range start end ?step?` | `range 1 10` → `1 2 3 4 5 6 7 8 9` |
| Slice | `$list(start..end)` | `$mylist(2..5)` |
| Pattern matching | `match val { pat body }` | See below |
| Lambda | `{args { body }}` | `{x { expr {$x * 2} }}` |
| List map/filter/reduce | `lmap`, `lfilter`, `lreduce` | `lmap $list {x { expr {$x + 1} }}` |
| Heredoc | `<<TAG ... TAG` | Multi-line strings |
| Typed declarations | `declare var : type` | `declare mode : enum {a b c}` |
| Static validation | `validate` pass at load time | Catches typos, type errors, arity |
| Command manifests | `manifest { ... }` | Declare external command signatures |

### Removed (from Tcl)

| Removed | Replacement |
|---------|-------------|
| `global` | Lexical scoping (reads walk up automatically) |
| `upvar` | `outer set var value` |
| `uplevel` | Lexical scoping makes it unnecessary |
| `namespace` | `context` blocks |
| `variable` | `declare` |
| `array` commands | Dict commands + accessor syntax |
| `regexp`/`regsub` | Via bridge commands (host app provides) |
| `open`/`close`/`read`/`gets` | Via bridge commands |
| `exec` | Via bridge commands |
| `trace` | Not needed |
| `interp` | No sub-interpreters |

## Core syntax

### Commands

Everything is a command. The first word is the command name, the rest
are arguments:

```tcl
command arg1 arg2 arg3
```

### Substitution

- `$var` — variable substitution
- `$var(index)` — accessor (list index or dict key)
- `$var.property` — property access
- `$ctx::var` — context variable access
- `[command args]` — command substitution (result replaces the brackets)
- `\n`, `\t`, `\\` — backslash substitution

### Quoting

- `{text}` — no substitution (literal)
- `"text"` — substitution happens inside
- `%{ key: val }` — dict literal (substitution in values)
- `%[ val, val ]` — list literal (substitution in values)

### Comments

```tcl
# This is a comment (must be at start of command)
set x 42  ;# Inline comment after semicolon
```

## Variables

### Basic assignment

```tcl
# Classic Tcl form
set x 42
set name "hello"

# Modern form (= is optional for single vars, required for destructuring)
set x = 42
set name = "hello"
```

### Destructuring

```tcl
# List destructuring (= required)
set a, b, c = [list 1 2 3]
puts $b   ;# → 2

# Dict destructuring (braces for dict pattern)
set {name, age} = [dict create name "alice" age 30]
puts $name  ;# → "alice"

# Swap
set a, b = [list $b $a]
```

### Scoping

Reads walk up the scope chain. Writes are local by default:

```tcl
set x 10
proc foo {} {
    puts $x          ;# reads parent's x → 10
    set y 20         ;# local to foo
    outer set x 30   ;# writes to parent scope
}
foo
puts $x              ;# → 30
```

### Context blocks

Named scopes for organizing related state:

```tcl
context editor {
    declare mode : enum {normal insert visual command}
    declare modified : bool

    set mode normal
    set modified false

    proc save {} {
        if {$modified} { buffer write }
    }
}

# Access from outside
puts $editor::mode
editor::save
set editor::mode insert
```

## Data types

### Strings

The default type. Everything can be represented as a string:

```tcl
set s "hello world"
string length $s        ;# → 11
string toupper $s       ;# → "HELLO WORLD"
string range $s 0 4     ;# → "hello"
string trim "  hi  "    ;# → "hi"
```

### Numbers

```tcl
set n 42
set f 3.14
expr {$n + 1}           ;# → 43
expr {$f * 2}           ;# → 6.28
incr n                  ;# n is now 43
```

### Booleans

```tcl
set flag true
# Accepted values: true/false, on/off, yes/no, 1/0
if {$flag} { puts "yes" }
```

### Lists

```tcl
# Classic
set l [list a b c d e]

# Structured literal
set l %[ "a", "b", "c", "d", "e" ]

# Access
$l(0)                   ;# → "a"
$l.len                  ;# → 5
$l(1..3)                ;# → "b c d"

# Operations
lappend l "f"
lsort $l
lmap $l {x { string toupper $x }}
lfilter $l {x { expr {$x ne "c"} }}
```

### Dicts

```tcl
# Classic
set d [dict create name "kairn" version 1]

# Structured literal
set d %{ name: "kairn", version: 1 }

# Access
$d("name")              ;# → "kairn"
$d.len                  ;# → 2
$d.keys                 ;# → "name version"

# Operations
dict set d author "cyril"
dict for {k v} $d { puts "$k = $v" }
```

## Control flow

### Conditionals

```tcl
if {$x > 0} {
    puts "positive"
} elseif {$x == 0} {
    puts "zero"
} else {
    puts "negative"
}
```

### Loops

```tcl
while {$i < 10} {
    incr i
}

foreach item $mylist {
    puts $item
}

for {set i 0} {$i < 10} {incr i} {
    puts $i
}
```

### Pattern matching

```tcl
match $value {
    "ok" result   { puts "Success: $result" }
    "err" message { puts "Failed: $message" }
    _             { puts "Unknown" }
}
```

### Error handling

```tcl
try {
    buffer save $path
} on error {msg} {
    puts "Failed: $msg"
} finally {
    cleanup
}
```

## Procedures

```tcl
proc greet {name} {
    puts "Hello, $name!"
}

# With type annotations
proc add {a:int b:int} : int {
    return [expr {$a + $b}]
}

# With default values
proc open {path {mode "read"}} {
    ...
}

# Lambda
set double {x { expr {$x * 2} }}
lmap [range 1 5] $double   ;# → 2 4 6 8
```

## Pipe operator

Pipes pass the result of the left side as the last argument to the right:

```tcl
# These are equivalent:
string toupper [string trim [lindex $list 0]]
$list(0) | string trim | string toupper

# Data processing pipeline:
files "." | lfilter {f { string match "*.rs" $f }} | lsort | foreach f { puts $f }
```

## Structured literals

### Dict literals

```tcl
set config %{
    editor: %{
        keymap: "vi",
        tab-width: 4,
        auto-save: true
    },
    languages: %[
        %{ name: "rust", ext: ".rs" },
        %{ name: "java", ext: ".java" }
    ]
}

# Access nested data
$config("editor")("keymap")        ;# → "vi"
$config("languages")(0)("name")    ;# → "rust"
```

### Optional chaining

```tcl
# Returns empty string if any key is missing
$config("editor")?("theme")?

# With default
$config("editor")?("theme")? ?? "gruvbox"
```

## Type system

Types are optional. Untyped variables accept anything (classic Tcl).
Typed variables are checked at assignment time.

### Declarations

```tcl
context config {
    declare mode      : enum {vi emacs classic}
    declare tab-width : int
    declare auto-save : bool
    declare theme     : string
    declare path      : string?          ;# nullable
    declare tags      : list:string
    declare user      : record {name:string age:int}
}

set config::mode vi          ;# OK
set config::mode "bogus"     ;# ERROR: not a valid mode
set config::tab-width "four" ;# ERROR: expected int
```

### Available types

| Type | Accepts |
|------|---------|
| `string` | Anything |
| `int` | Integer values |
| `float` | Numeric values |
| `bool` | true/false/on/off/yes/no/1/0 |
| `enum {a b c}` | One of the listed values |
| `string?`, `int?` | Value or empty (nullable) |
| `list` | Any list |
| `list:int` | List of integers |
| `list:record{...}` | List of typed records |
| `dict` | Any dict |
| `record {k:type ...}` | Dict with known typed keys |

## Static validation

Scripts are validated at load time. The validator catches:

- Wrong number of arguments to procs
- Unknown command names (with "did you mean?" suggestions)
- Undefined variable references
- Type mismatches on declared variables
- Dead code (unreachable after return/break)
- Variable shadowing
- Non-exhaustive switch/match on enums

```tcl
# These errors are caught at load time, not runtime:
proc foo {} { putz "hello" }     ;# ERROR: unknown command "putz"
set config::mode "bogus"         ;# ERROR: not a valid mode
save-file                        ;# ERROR: wrong # args
```

## Heredoc strings

```tcl
# With substitution
set msg <<END
Hello $name,
You have $count items.
END

# Without substitution
set raw <<'END'
No $substitution here.
Literal [brackets] too.
END
```

## Command manifests

External commands (provided by the host application) can be declared
for validation:

```tcl
manifest {
    command buffer {
        save ?path:string?
        list
        modified ?bufid:string?
    }
    command editor {
        goto {line:int col:int}
        insert {text:string}
    }
}
```

This allows the validator to check calls to host-provided commands
at load time.
