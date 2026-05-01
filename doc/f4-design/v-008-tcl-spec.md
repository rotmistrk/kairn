# v-008 — rusticle Spec: Tcl Subset Interpreter

*rusticle* — Rust + Tcl + particle. A Tcl-compatible scripting language
with lexical scoping, typed declarations, structured literals, accessor
syntax, and load-time static analysis.

## Purpose

A scripting language interpreter for configuration, keybinding, and
automation. Reusable in any Rust application, not tied to kairn or TUI.

Improvements over standard Tcl:
- Lexical scoping (no `global`/`upvar`/`uplevel`)
- `context` blocks for organizing related state
- Typed variable declarations with load-time validation
- Structured literals: `%{ key: value }` and `%[ item, item ]`
- Accessor syntax: `$var(index)`, `$var("key")`, `$var.len`
- Command manifests for external command validation
- Dual-representation values (string interface, typed internals)
- Static analysis: type inference, dead code, shadowing, exhaustiveness

## Dependencies

None. Pure Rust, no TUI, no async.

## Core types

### TclValue — dual representation

```rust
#[derive(Clone, Debug)]
pub enum TclValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<TclValue>),
    Dict(Vec<(String, TclValue)>),
}

impl TclValue {
    fn as_str(&self) -> Cow<str>;
    fn as_int(&self) -> Result<i64, TclError>;
    fn as_float(&self) -> Result<f64, TclError>;
    fn as_bool(&self) -> Result<bool, TclError>;
    fn as_list(&self) -> Result<&[TclValue], TclError>;
}
```

Invariant: `eval(to_string(value)) == value` for all variants.

### TclError

```rust
pub struct TclError {
    pub message: String,
    pub code: ErrorCode,
    pub location: Option<Location>,
}

pub enum ErrorCode {
    Error,
    Return(TclValue),
    Break,
    Continue,
}

pub struct Location {
    pub source: String,
    pub line: usize,
    pub col: usize,
}
```

### Interpreter

```rust
pub struct Interpreter { ... }

impl Interpreter {
    fn new() -> Self;
    fn eval(&mut self, script: &str) -> Result<TclValue, TclError>;
    fn eval_source(&mut self, script: &str, source: &str) -> Result<TclValue, TclError>;
    fn set_var(&mut self, name: &str, value: TclValue);
    fn get_var(&self, name: &str) -> Option<&TclValue>;
    fn register_command(&mut self, name: &str, cmd: Box<dyn TclCommand>);
    fn load_manifest(&mut self, manifest: Manifest);
    fn validate(&self, script: &str) -> ValidationResult;
}

pub trait TclCommand: Send {
    fn call(&mut self, interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError>;
}
```

## Syntactic enhancements

### Structured literals

`%{ }` for dicts, `%[ ]` for lists. Parser-level — builds typed values
directly, no runtime re-parsing.

```tcl
set mydict %{ name: "kairn", version: 1, active: true }
set mylist %[ "alpha", "beta", "gamma" ]

# Nested
set config %{
    editor: %{
        keymap: "vi",
        tab-width: 4,
        auto-save: true
    },
    languages: %[
        %{ name: "rust", ext: ".rs", lsp: "rust-analyzer" },
        %{ name: "java", ext: ".java", lsp: "jdtls" }
    ]
}
```

Rules inside `%{ }`:
- `key: value` pairs
- Separated by commas or newlines
- Trailing commas allowed
- Keys are unquoted words or `"quoted strings"`
- Values: strings, numbers, `true`/`false`, nested `%{}`, nested `%[]`
- `$var` substitution in values (not in keys)

Rules inside `%[ ]`:
- Values separated by commas or newlines
- Trailing commas allowed
- Same value types as dict values

Standard Tcl `[list ...]` and `[dict create ...]` still work. The `%`
literals are syntactic sugar for common cases.

### Accessor syntax

Parser-level desugaring for natural container access.

```tcl
# List access
$mylist(0)              ;# → [lindex $mylist 0]
$mylist.len             ;# → [llength $mylist]

# Dict access
$mydict("name")         ;# → [dict get $mydict name]
$mydict.len             ;# → [dict size $mydict]
$mydict.keys            ;# → [dict keys $mydict]
$mydict.values          ;# → [dict values $mydict]

# Chained access
$config("editor")("keymap")        ;# → "vi"
$config("languages")(0)("name")    ;# → "rust"

# Variable substitution in index
set i 2
$mylist($i)             ;# → [lindex $mylist $i]
```

Properties available on all values:

| Property | Desugars to |
|----------|-------------|
| `.len` | `llength` (list) or `dict size` (dict) or `string length` (string) |
| `.keys` | `dict keys` |
| `.values` | `dict values` |
| `.type` | Returns: "string", "int", "float", "bool", "list", "dict" |

The parser detects `(` and `.` after `$varname` and rewrites before
evaluation. `$var(key)` replaces Tcl's old array syntax (arrays are
replaced by dicts).

### Pipe operator

```tcl
# Instead of nested commands:
string toupper [string trim [lindex $mylist 0]]

# Pipe — reads left to right:
$mylist(0) | string trim | string toupper

# Multi-step data processing:
files "." | lsort | foreach f { puts $f }

# With lambda:
%[1, 2, 3, 4, 5] | lmap {x { expr {$x * 2} }} | lsort
```

Parser rewrites `a | b` to `b [a]`. Zero runtime cost.

### Destructuring assignment

```tcl
# List destructuring
set [a, b, c] [list 1 2 3]
puts $b   ;# → 2

# Dict destructuring
set {name, age} [dict create name "alice" age 30]
puts $name  ;# → "alice"

# In foreach
foreach {key, value} $mydict {
    puts "$key = $value"
}

# Swap
set [a, b] [list $b $a]
```

### Optional chaining and null coalescing

```tcl
# Standard: crashes if key doesn't exist
dict get $config "editor" "theme"

# Optional chain: returns empty string if any step fails
$config("editor")?("theme")?

# Null coalescing: default value if empty
$config("editor")?("theme")? ?? "gruvbox"

# Chained with property
$data("users")?(0)?("name")? ?? "anonymous"
```

`?` after accessor = return empty on missing. `??` = null coalescing.
Both are parser-level rewrites to `catch` + fallback.

### Range expressions and slicing

```tcl
# Generate a list
set nums [range 1 10]        ;# → 1 2 3 4 5 6 7 8 9
set evens [range 0 20 2]     ;# → 0 2 4 6 8 10 12 14 16 18

# Slice with range
$mylist(2..5)                 ;# → [lrange $mylist 2 4]
$mylist(3..)                  ;# → from index 3 to end
$mylist(..5)                  ;# → first 5 elements
```

### Pattern matching

```tcl
match $value {
    "ok" result   { puts "Success: $result" }
    "err" message { puts "Failed: $message" }
    _             { puts "Unknown" }
}

# With type patterns
match $item {
    int n    { expr {$n + 1} }
    string s { string length $s }
    list l   { llength $l }
    dict d   { dict size $d }
}
```

Like `switch` but with destructuring and variable binding. `_` is the
wildcard. Type patterns check the value's internal representation.

### Lambda / anonymous procs

```tcl
# Named lambda
set double {x { expr {$x * 2} }}

# Use with lmap (list map)
lmap $mylist {x { expr {$x * 2} }}
;# → applies lambda to each element, returns new list

# Filter
lfilter $mylist {x { expr {$x > 3} }}

# Reduce
lreduce $mylist 0 {acc x { expr {$acc + $x} }}
```

Lambdas are two-element lists: `{args body}`. They capture the defining
scope (lexical closure).

### Heredoc strings

```tcl
set html <<END
    <div class="user">
        <h1>${name}</h1>
        <p>${count} items</p>
    </div>
END

# No substitution variant (like Tcl braces):
set raw <<'END'
    No $substitution here
    No [commands] either
END
```

`<<TAG` performs variable and command substitution. `<<'TAG'` is literal
(no substitution). Leading whitespace is stripped based on the indentation
of the closing tag.

### Try / on / finally

```tcl
try {
    buffer save $path
} on error {msg} {
    puts "Save failed: $msg"
} finally {
    cleanup
}
```

Cleaner than `catch` + manual error code checking. `on error` catches
errors. `finally` always runs. Multiple `on` clauses can match different
error types (future: typed errors).

### Summary of syntax enhancements

All enhancements are parser-level rewrites or thin builtins. The core
interpreter remains simple.

| Feature | Parser | Runtime | Effort |
|---------|:------:|:-------:|--------|
| Structured literals `%{}` `%[]` | Rewrite | None | Small |
| Accessor `$var(i)` `$var.len` | Rewrite | None | Small |
| Pipe `\|` | Rewrite | None | Tiny |
| Destructuring `set [a,b]` | Rewrite | Small | Small |
| Optional chain `?` | Rewrite | None | Small |
| Null coalescing `??` | Rewrite | None | Tiny |
| Range `range 1 10` | None | Builtin | Tiny |
| Slice `$list(2..5)` | Rewrite | None | Small |
| Pattern matching `match` | New syntax | Builtin | Medium |
| Lambda `{x { body }}` | None | Small | Small |
| `lmap` / `lfilter` / `lreduce` | None | Builtins | Tiny |
| Heredoc `<<TAG` | New syntax | None | Small |
| Try/on/finally | New syntax | Builtin | Medium |

## Scoping model

### Lexical scoping

Reads walk up the scope chain. Writes are local by default.

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

`outer` can be chained: `outer outer set x 1` writes to grandparent.

### Context blocks

Named scopes with typed declarations.

```tcl
context editor {
    declare mode     : enum {normal insert visual command}
    declare modified : bool
    declare path     : string?

    set mode normal
    set modified false

    proc save {} {
        if {$modified} {
            buffer write $path
            set modified false
        }
    }
}

# Access from outside
puts $editor::mode
editor::save
```

Assignment to declared variables triggers type validation at both
load time (if value is static) and runtime (if value is dynamic).

## Type system

### Type declarations

```tcl
declare varname : type
```

| Type syntax | Validates |
|-------------|-----------|
| `string` | Anything |
| `int` | Integer |
| `float` | Number |
| `bool` | true/false/on/off/yes/no/1/0 |
| `enum {a b c}` | One of listed values |
| `string?`, `int?` | Nullable (empty = null) |
| `list` | Valid list |
| `list:int` | List of integers |
| `list:record{...}` | List of typed records |
| `dict` | Valid dict |
| `record {k:type ...}` | Dict with known keys and typed values |

### Proc signature types

```tcl
proc goto {line:int col:int} {
    editor goto $line $col
}

proc open {path:string {mode:enum{read write} "read"}} {
    ...
}
```

### Return type annotation

```tcl
proc get-count {} : int {
    return [llength $items]
}
```

## Static analysis

### Load-time validation

When a script is loaded, a validation pass runs before execution.

| Analysis | Severity | Example |
|----------|----------|---------|
| Proc arity | Error | `save` called with wrong arg count |
| Unknown command | Error | `bufer save` (typo) |
| Undefined variable | Warning | `$widht` never set in scope |
| Type mismatch (declared) | Error | `set mode "bogus"` on enum |
| Dead code | Warning | Proc never called |
| Unreachable code | Warning | Code after `return` |
| Shadowing | Warning | Local `set x` hides outer `x` |
| Non-exhaustive switch | Warning | Missing enum case |
| Manifest signature | Error | Wrong args to bridge command |

### Type inference (data flow)

Track types through assignments and known-return-type builtins:

```tcl
set x 42                        ;# inferred: int
set y [expr {$x + 1}]           ;# inferred: int (arithmetic)
set z [string length $x]        ;# inferred: int (string length)
puts [expr {$z + "hello"}]      ;# ERROR: can't add int and non-numeric
```

Known return types:
- `expr` with arithmetic → int or float
- `string length` → int
- `llength` → int
- `dict size` → int
- `lindex` → element type (if list is typed) or string
- `dict get` → value type (if dict is typed) or string

### Accessor chain validation

When the container has a typed declaration, accessor chains are fully
validated at load time:

```tcl
context config {
    declare users : list:record{name:string age:int}
}

puts $config::users(0)("name")    ;# OK — string
puts $config::users(0)("age")     ;# OK — int
puts $config::users(0)("email")   ;# ERROR: 'email' not in record
puts [expr {$config::users(0)("age") + 1}]  ;# OK — int arithmetic
```

### Validation API

```rust
pub struct ValidationResult {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
}

pub struct Diagnostic {
    pub message: String,
    pub location: Location,
    pub severity: Severity,
    pub suggestion: Option<String>,
}

pub enum Severity { Error, Warning }
```

### Command manifests

External commands described for validation:

```tcl
manifest {
    command buffer {
        save ?path:string?
        list
        modified ?bufid:string?
    }
    command editor {
        selection
        cursor
        insert {text:string}
        goto {line:int col:int}
    }
    command bind {keyspec:string script:string}
}
```

## Built-in commands

### Variables and control flow

| Command | Description |
|---------|-------------|
| `set var value` | Set variable in current scope |
| `unset var` | Remove variable |
| `outer set var value` | Set variable in parent scope |
| `if {cond} {body} ?elseif ...? ?else {body}?` | Conditional |
| `while {cond} {body}` | Loop |
| `foreach var list body` | Iterate list |
| `for {init} {cond} {step} {body}` | C-style loop |
| `break` | Exit loop |
| `continue` | Next iteration |
| `return ?value?` | Return from proc |
| `proc name args body` | Define procedure |
| `context name body` | Define named scope |
| `declare var : type` | Typed declaration (inside context) |
| `manifest body` | Declare external command signatures |

### Expressions

| Command | Description |
|---------|-------------|
| `expr {expression}` | `+ - * / % == != < > <= >= && \|\| !` |
| `incr var ?amount?` | Increment integer |

### Strings

| Command | Description |
|---------|-------------|
| `string length str` | Character count |
| `string range str first last` | Substring |
| `string match pattern str` | Glob match |
| `string map {old new ...} str` | Substitution |
| `string trim str ?chars?` | Trim |
| `string tolower str` / `toupper` | Case |
| `string first needle haystack` | Find |
| `format fmt args...` | Printf-style |
| `append var str` | Append to variable |

### Lists

| Command | Description |
|---------|-------------|
| `list args...` | Create list |
| `lindex list index` | Get element (also: `$list(index)`) |
| `llength list` | Length (also: `$list.len`) |
| `lappend var element` | Append |
| `lrange list first last` | Sublist |
| `lsearch list pattern` | Find |
| `lsort list` | Sort |
| `lset var index value` | Set element |
| `join list ?sep?` | Join to string |
| `split str ?sep?` | Split to list |
| `lmap list lambda` | Map: apply lambda to each element, return new list |
| `lfilter list lambda` | Filter: keep elements where lambda returns true |
| `lreduce list init lambda` | Reduce: fold list with accumulator |
| `range start end ?step?` | Generate integer list |

### Dicts

| Command | Description |
|---------|-------------|
| `dict create key val ...` | Create (also: `%{ k: v }`) |
| `dict get dict key` | Get (also: `$dict("key")`) |
| `dict set dictvar key value` | Set |
| `dict exists dict key` | Check key |
| `dict keys dict` | Keys (also: `$dict.keys`) |
| `dict values dict` | Values (also: `$dict.values`) |
| `dict size dict` | Size (also: `$dict.len`) |
| `dict for {k v} dict body` | Iterate |

### I/O and misc

| Command | Description |
|---------|-------------|
| `puts ?-nonewline? string` | Output (app-defined sink) |
| `source reader` | Evaluate script |
| `catch script ?resultvar?` | Catch errors |
| `error message` | Raise error |
| `info commands ?pattern?` | List commands |
| `info vars ?pattern?` | List variables |
| `info procs ?pattern?` | List procedures |
| `after ms script` | Timer (event loop integration) |
| `switch value {pattern body ...}` | Pattern matching |
| `match value {pattern ?var? body ...}` | Destructuring pattern match |
| `try body ?on error {var} body? ?finally body?` | Structured error handling |

### NOT included

| Omitted | Reason |
|---------|--------|
| `global`, `upvar`, `uplevel` | Replaced by lexical scoping |
| `namespace`, `variable` | Replaced by `context` + `declare` |
| `regexp`/`regsub` | Via bridge command (Rust regex) |
| `open`/`close`/`read`/`gets` | Via bridge commands |
| `exec` | Via bridge commands |
| `trace`, `interp` | Not needed |
| Array syntax (`set arr(key)`) | Replaced by dict + accessor syntax |

## Implementation

### Module structure

```
rusticle/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── value.rs          — TclValue, conversions, equality
    ├── error.rs          — TclError, ErrorCode, Location
    ├── parser.rs         — command parsing, substitution, %{} %[] literals
    ├── interpreter.rs    — eval, scope chain, command dispatch
    ├── builtins.rs       — all built-in commands
    ├── context.rs        — context blocks, typed declarations
    ├── types.rs          — TypeDecl, type checking, type inference
    ├── validate.rs       — load-time validation pass
    └── manifest.rs       — command manifest loading
```

### Size estimate

| Module | Lines (est.) |
|--------|-------------|
| value.rs | 250–350 |
| error.rs | 100–150 |
| parser.rs | 600–900 (literals, accessors, pipes, heredoc, destructuring) |
| interpreter.rs | 500–700 |
| builtins.rs | 1000–1400 (includes lmap/lfilter/lreduce, match, try, range) |
| context.rs | 200–300 |
| types.rs | 300–400 |
| validate.rs | 500–700 |
| manifest.rs | 150–200 |
| **Total** | **~3,600–5,100** |

### Testing strategy

Unit tests per module. Integration tests for end-to-end scripts.

```rust
#[test]
fn structured_literal_dict() {
    let mut interp = Interpreter::new();
    interp.eval(r#"set d %{ name: "kairn", ver: 1 }"#).unwrap();
    let result = interp.eval(r#"return $d("name")"#).unwrap();
    assert_eq!(result.as_str(), "kairn");
}

#[test]
fn structured_literal_nested() {
    let mut interp = Interpreter::new();
    interp.eval(r#"
        set cfg %{
            items: %[ "a", "b", "c" ]
        }
    "#).unwrap();
    let result = interp.eval(r#"return $cfg("items")(1)"#).unwrap();
    assert_eq!(result.as_str(), "b");
}

#[test]
fn accessor_len() {
    let mut interp = Interpreter::new();
    interp.eval("set xs [list 1 2 3 4 5]").unwrap();
    let result = interp.eval("return $xs.len").unwrap();
    assert_eq!(result.as_int().unwrap(), 5);
}

#[test]
fn pipe_operator() {
    let mut interp = Interpreter::new();
    interp.eval(r#"set x [list "  hello  " "  world  "]"#).unwrap();
    let result = interp.eval(r#"$x(0) | string trim | string toupper"#).unwrap();
    assert_eq!(result.as_str(), "HELLO");
}

#[test]
fn destructuring_list() {
    let mut interp = Interpreter::new();
    interp.eval("set [a, b, c] [list 10 20 30]").unwrap();
    let result = interp.eval("return $b").unwrap();
    assert_eq!(result.as_int().unwrap(), 20);
}

#[test]
fn optional_chain_missing_key() {
    let mut interp = Interpreter::new();
    interp.eval(r#"set d %{ name: "kairn" }"#).unwrap();
    let result = interp.eval(r#"return [$d("missing")? ?? "default"]"#).unwrap();
    assert_eq!(result.as_str(), "default");
}

#[test]
fn range_generates_list() {
    let mut interp = Interpreter::new();
    let result = interp.eval("return [range 1 5]").unwrap();
    assert_eq!(result.as_str(), "1 2 3 4");
}

#[test]
fn lmap_with_lambda() {
    let mut interp = Interpreter::new();
    let result = interp.eval(r#"
        lmap [list 1 2 3] {x { expr {$x * 10} }}
    "#).unwrap();
    assert_eq!(result.as_str(), "10 20 30");
}

#[test]
fn pattern_match_with_binding() {
    let mut interp = Interpreter::new();
    let result = interp.eval(r#"
        set val "ok"
        match $val {
            "ok"  { return "success" }
            "err" { return "failure" }
            _     { return "unknown" }
        }
    "#).unwrap();
    assert_eq!(result.as_str(), "success");
}

#[test]
fn try_catch_finally() {
    let mut interp = Interpreter::new();
    let result = interp.eval(r#"
        set log ""
        try {
            error "boom"
        } on error {msg} {
            append log "caught:$msg"
        } finally {
            append log ",cleaned"
        }
        return $log
    "#).unwrap();
    assert_eq!(result.as_str(), "caught:boom,cleaned");
}

#[test]
fn heredoc_with_substitution() {
    let mut interp = Interpreter::new();
    interp.eval("set name world").unwrap();
    let result = interp.eval(r#"
        set msg <<END
hello $name
END
        return $msg
    "#).unwrap();
    assert_eq!(result.as_str().trim(), "hello world");
}

#[test]
fn lexical_scoping() {
    let mut interp = Interpreter::new();
    interp.eval("set x 10").unwrap();
    interp.eval("proc foo {} { return $x }").unwrap();
    let result = interp.eval("foo").unwrap();
    assert_eq!(result.as_str(), "10");
}

#[test]
fn context_type_validation() {
    let mut interp = Interpreter::new();
    interp.eval("context cfg { declare mode : enum {a b c} }").unwrap();
    interp.eval("set cfg::mode a").unwrap();
    let err = interp.eval("set cfg::mode z").unwrap_err();
    assert!(err.message.contains("not a valid"));
}

#[test]
fn validate_catches_typo() {
    let interp = Interpreter::new();
    let result = interp.validate("proc foo {} { putz hello }");
    assert!(result.errors.iter().any(|d| d.message.contains("putz")));
}

#[test]
fn validate_type_inference() {
    let interp = Interpreter::new();
    let result = interp.validate(r#"
        set x 42
        set y [expr {$x + "hello"}]
    "#);
    assert!(result.errors.iter().any(|d| d.message.contains("non-numeric")));
}
```
