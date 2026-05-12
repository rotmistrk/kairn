# Build & Test Integration

## Overview

`:build` and `:test` commands that run project-specific build/test commands,
parse output for errors, and display results in a ResultsView (tool panel).
Navigation identical to grep results.

## Configuration Priority

1. `.kairn/init` in workspace root (explicit override)
2. `~/.kairn/config.tcl` (user defaults per project type)
3. Auto-detection from marker files in workspace root

### .kairn/init format
```
build = cargo build --message-format=short 2>&1
test = cargo test --workspace 2>&1
test-file = cargo test --lib {file} 2>&1
test-at-cursor = cargo test {test_name} 2>&1
```

### ~/.kairn/config.tcl (user home config)
```tcl
# Default build commands by project type
set build(cargo) "cargo build 2>&1"
set build(make) "make 2>&1"
set build(gradle) "./gradlew build 2>&1"
set test(cargo) "cargo test --workspace 2>&1"
set test(make) "make test 2>&1"
```

### Auto-detection (scan workspace root, first match wins)

| Priority | Marker File | Build Command | Test Command |
|----------|-------------|---------------|--------------|
| 1 | Makefile / GNUmakefile | `make` | `make test` |
| 2 | Cargo.toml | `cargo build` | `cargo test --workspace` |
| 3 | go.mod | `go build ./...` | `go test ./...` |
| 4 | gradlew | `./gradlew build` | `./gradlew test` |
| 5 | build.gradle / build.gradle.kts | `gradle build` | `gradle test` |
| 6 | pom.xml | `mvn compile` | `mvn test` |
| 7 | CMakeLists.txt | `cmake --build build` | `ctest --test-dir build` |
| 8 | package.json | `npm run build` | `npm test` |
| 9 | build.xml | `ant` | `ant test` |
| 10 | configure.ac / Makefile.am | `make` | `make check` |
| 11 | meson.build | `meson compile -C build` | `meson test -C build` |
| 12 | BUILD / WORKSPACE | `bazel build //...` | `bazel test //...` |

## Implementation

### Shared Infrastructure (reuse with grep)

Rename `GrepState` → `TaskOutput`:
```rust
pub struct TaskOutput {
    pub entries: Mutex<Vec<ResultEntry>>,
    pub done: Mutex<bool>,
    pub error: Mutex<Option<String>>,
    pub exit_code: Mutex<Option<i32>>,
}
```

Both grep and build use: `TaskOutput` + `ResultsView` + waker + handler drain.

### Error Parsing

Parse each output line as `file:line:col: text` or `file:line: text`.
Additionally, handle format-specific patterns:

**Rust (cargo):**
```
error[E0123]: message
  --> src/file.rs:42:5
```
Multi-line: accumulate until `-->` line, use that as the location.

**GCC/Clang:**
```
src/file.cpp:42:5: error: message
```
Standard `file:line:col: text` — works with generic parser.

**Go:**
```
./file.go:42:5: message
```
Standard format.

**Generic fallback:**
Any line matching `path:number:text` or `path:number:number:text`.

### Parser module: `src/build_parse.rs`
```rust
pub fn parse_error_line(line: &str, root: &Path) -> Option<ResultEntry>
```
Try Rust multi-line first, then generic. Return None for non-error lines.

### Build runner: `src/build.rs`
```rust
pub fn run_build_async(cmd: &str, root: &Path, waker: Waker) -> Arc<TaskOutput>
```
- Spawn `sh -c "{cmd}"` with stdout+stderr merged
- Parse each line, push ResultEntry batches
- Wake after each batch
- Store exit code when done

### Handler integration

In `handle_command`:
- Same drain pattern as grep: check `state.build_pending`, drain entries, append to ResultsView
- `:build` → detect command, open ResultsView("build"), spawn
- `:test` → detect command, open ResultsView("test"), spawn
- `:test-file` → substitute `{file}` with current file path
- `:test-at-cursor` → substitute `{test_name}` with function name at cursor

### ResultsView enhancements for build

- Status line shows exit code when done: "✓ Build succeeded" / "✗ Build failed (exit 1)"
- Color: green for 0, red for non-zero
- `n`/`p` navigate errors (skip non-error lines if we add them later)

### Keybindings

- `:build` — run build
- `:test` — run all tests
- `:test-file` — test current file
- `:test-at-cursor` — test function at cursor
- `:next-error` / `:prev-error` — navigate build errors (already exist as commands)

## Files to Create/Modify

- `src/build_detect.rs` — auto-detect build system
- `src/build_parse.rs` — error line parser
- `src/build.rs` — rewrite: async runner using TaskOutput
- `src/grep.rs` — rename GrepState → TaskOutput, move to shared module
- `src/task_output.rs` — shared TaskOutput struct (used by grep + build)
- `src/handler_exec.rs` — `:build`, `:test` commands
- `src/handler.rs` — drain build_pending same as grep_pending
- `src/config.rs` — read .kairn/init and ~/.kairn/config.tcl

## Testing

1. In kairn workspace: `:build` should run `make` (Makefile present) or `cargo build`
2. Introduce a compile error → error appears in results, Enter navigates to it
3. `:test` runs tests, failures shown as navigable entries
4. Create `.kairn/init` with custom command → verify it's used
5. Project with no recognized build system → show "No build command configured" error

## Constraints

- NO external tool dependencies for parsing (pure Rust)
- Errors MUST be visible (status bar + results view)
- Async: UI never blocks during build
- Wake pipe ensures results appear immediately
