# kairn mini-IDE — Design Specification

## Overview

Transform kairn from a code viewer into a minimal IDE with editing, building, testing, navigation, and AI-assisted refactoring. Target languages: Java (primary), Go, Rust, TypeScript.

## Branch Strategy

- Branch: `feature/mini-ide` (based on `master`)
- Regularly merge from `master` to pick up UI fixes, keybinding changes, etc.
- Each phase is a set of commits on this branch
- Merge to master when stable

## Architecture

```
kairn-core (existing)
├── kairn-editor        — vi/emacs text editing in main panel tabs
├── kairn-nav           — import-based navigation (no LSP needed)
│   ├── java-nav        — parse Java imports → file paths
│   ├── go-nav          — parse Go imports
│   ├── rust-nav        — parse Rust use statements
│   └── ts-nav          — parse TS/JS imports
├── kairn-lsp           — LSP client (Phase 3)
├── kairn-runner        — build/test/run framework
│   ├── java-runner     — Maven/Gradle, JUnit, main method detection
│   ├── go-runner       — go build/test
│   └── rust-runner     — cargo build/test
├── kairn-search        — ripgrep-style file content search
└── kairn-kiro-refactor — apply kiro AI responses as patches
```

## Phase 1: Editor + Search + Import Navigation (2-3 weeks)

### 1a. Vi Editing Mode

Main panel gets a new mode: **Edit**. Opened via `e` in cursor mode or Enter on a file.

**Data model:**
- `EditBuffer`: lines of text, cursor position, undo stack
- `EditMode`: Normal, Insert, Visual, Command (`:w`, `:q`)
- Modified indicator in title bar (`[+]`)

**Vi commands (minimum viable):**
- Movement: `h/j/k/l`, `w/b/e`, `0/$`, `gg/G`, `Ctrl-D/U`
- Insert: `i/a/o/O/I/A`
- Delete: `x/dd/dw/d$`
- Change: `cw/cc/c$`
- Undo/redo: `u/Ctrl-R`
- Visual: `v/V` (reuse existing)
- Save: `:w` / Ctrl-S
- Quit: `:q` / Esc back to view mode

**Tab model:**
- Default tab: view mode (current behavior, read-only)
- Edit tab: opened with `e`, has undo stack, save capability
- Tab bar in main panel shows: `view | edit:main.rs [+]`

### 1b. Search in Files

`Ctrl-Shift-F` or configurable key opens search panel (tab in terminal panel area).

- Input: query string (regex optional)
- Scope: workspace or subdirectory
- Results: file:line:content, navigable
- Enter on result → opens file at that line in main panel
- Uses `grep` crate or shells out to `rg` if available

### 1c. Import-Based Navigation

**Java:**
```java
import com.example.service.UserService;
// → resolve to src/main/java/com/example/service/UserService.java
// → or src/main/kotlin/...
```

**Commands:**
- `gd` (go to definition) — on a class name, jump to its file
- `gr` (go to references) — find all files that import this class
- Results shown in a "references" tab in terminal panel

**Implementation:**
- Parse imports with regex (fast, no AST needed)
- Build import index on startup (background thread)
- Refresh on file save
- For Java: map package.Class → file path using source roots

**"What uses this / what this uses" panel:**
- Toggle with a key when viewing a file
- Shows two lists: dependencies (imports) and dependents (who imports this)

### 1d. Keyboard Layout Config

```json
{
  "editor_keymap": "vi"  // or "emacs" or "standard"
}
```

Vi is default and first implemented. Emacs/standard are future.

## Phase 2: Build / Run / Test (2-3 weeks)

### 2a. Java Runtime Config

```json
{
  "java": {
    "jdk_home": "/usr/lib/jvm/java-21",
    "source_roots": ["src/main/java", "src/test/java"],
    "classpath_command": "mvn dependency:build-classpath -q -DincludeScope=runtime -Dmdep.outputFile=/dev/stdout",
    "build_command": "mvn compile -q",
    "test_command": "mvn test",
    "main_args": {}
  }
}
```

`classpath_command` is key — runs a shell command that returns the classpath. This makes it work with Maven, Gradle, or any build tool.

### 2b. Run Main Method

- Detect `public static void main(String[] args)` in current file
- Or detect any `public static void methodName(String[])` and offer choice
- Prompt for args (saved per class between runs)
- Spawn in terminal tab: `java -cp $CLASSPATH com.example.Main args...`
- Output captured in dedicated terminal tab

### 2c. Test Runner

- Detect test files (JUnit annotations, file naming)
- Run single test, test class, or all tests
- Parse JUnit XML output or Maven surefire output
- Results tree in terminal panel tab:
  ```
  ✅ UserServiceTest (3/3)
    ✅ testCreate
    ✅ testUpdate  
    ❌ testDelete — AssertionError: expected 0 but was 1
  ```
- Navigate to failed test source
- Show test output on selection

### 2d. Build with Error Navigation

- Run build command, capture stderr
- Parse error format: `file:line:col: message`
- Error list in terminal panel tab
- `F8` / `Shift-F8` — next/prev error
- Jump to error location in editor
- Inline error markers in gutter (red dot)

## Phase 3: LSP + Kiro Refactoring (3-4 weeks)

### 3a. LSP Client

Connect to language servers:
- Java: Eclipse JDT Language Server (jdtls)
- Go: gopls
- Rust: rust-analyzer
- TypeScript: typescript-language-server

**Capabilities used:**
- `textDocument/definition` — go to definition
- `textDocument/references` — find usages
- `textDocument/completion` — auto-complete
- `textDocument/publishDiagnostics` — inline errors
- `textDocument/rename` — rename symbol
- `textDocument/formatting` — format file

**UI:**
- Completion popup (overlay near cursor)
- Diagnostics in gutter + underline
- Rename: inline rename with preview

### 3b. Kiro-as-Refactoring-Engine

**Flow:**
1. Select code in editor (`v`/`V`)
2. Press hotkey (e.g., `Ctrl-K r` for refactor)
3. Prompt appears: "Describe the refactoring:"
4. User types: "extract this into a method called processUser"
5. Kiro processes, returns modified code
6. **Diff view** shows before/after
7. User accepts (`y`) or rejects (`n`)
8. On accept: apply patch to editor buffer

**Auto-context sent to kiro:**
- File path and language
- Full file content (or relevant section)
- Import context
- Error message (if triggered from error navigation)
- Test output (if triggered from test failure)

### 3c. Smart Actions

Context-aware actions based on cursor position:
- On error → "Fix this error"
- On test failure → "Fix this test"
- On import → "Go to definition" (pre-LSP: import nav)
- On method → "Generate test for this method"
- On class → "Explain this class"

## Plugin Interface

```rust
pub trait LanguagePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn file_extensions(&self) -> &[&str];
    
    // Navigation (pre-LSP)
    fn resolve_import(&self, import: &str, workspace: &Path) -> Option<PathBuf>;
    fn find_importers(&self, file: &Path, workspace: &Path) -> Vec<PathBuf>;
    
    // Build/Run/Test
    fn build_command(&self, config: &Value) -> Option<String>;
    fn test_command(&self, config: &Value, target: TestTarget) -> Option<String>;
    fn run_command(&self, config: &Value, class: &str, args: &[String]) -> Option<String>;
    fn parse_errors(&self, output: &str) -> Vec<Diagnostic>;
    fn parse_test_results(&self, output: &str) -> Vec<TestResult>;
    
    // Detection
    fn detect_main_methods(&self, content: &str) -> Vec<MainMethod>;
    fn detect_test_methods(&self, content: &str) -> Vec<TestMethod>;
}
```

## File Structure (new modules)

```
src/
├── editor/
│   ├── mod.rs          — EditBuffer, EditMode
│   ├── vi.rs           — Vi command handling
│   ├── undo.rs         — Undo/redo stack
│   └── save.rs         — File save with backup
├── nav/
│   ├── mod.rs          — Navigation trait, index
│   ├── java.rs         — Java import resolution
│   ├── go.rs           — Go import resolution
│   ├── rust.rs         — Rust use resolution
│   └── ts.rs           — TypeScript import resolution
├── runner/
│   ├── mod.rs          — Runner trait, output parsing
│   ├── java.rs         — Java build/test/run
│   ├── go.rs           — Go build/test/run
│   └── rust.rs         — Rust/cargo build/test/run
├── search/
│   └── mod.rs          — File content search (ripgrep-style)
├── lsp/
│   ├── mod.rs          — LSP client, message handling
│   ├── completion.rs   — Completion popup
│   └── diagnostics.rs  — Inline diagnostics
└── refactor/
    └── mod.rs          — Kiro response → patch application
```

## Key Bindings (editor mode)

| Key | Vi Mode | Action |
|-----|---------|--------|
| `e` | cursor mode | Open file in editor |
| `i/a/o` | normal | Enter insert mode |
| `Esc` | insert | Back to normal |
| `:w` | normal | Save |
| `:q` | normal | Close editor tab |
| `gd` | normal | Go to definition |
| `gr` | normal | Find references |
| `Ctrl-Space` | insert | Trigger completion |
| `F8` | any | Next error |
| `Shift-F8` | any | Previous error |
| `Ctrl-K r` | visual | Kiro refactor |
| `Ctrl-K e` | on error | Kiro fix error |
| `Ctrl-K t` | on method | Kiro generate test |

## Success Criteria

1. Can edit a Java file with vi keybindings, save, build, see errors, navigate to error
2. Can run a test, see failure, navigate to test, fix it, re-run
3. Can jump between classes using imports (no LSP)
4. Can select code, send to kiro, apply the response as a patch
5. Can search across all files and navigate to results
6. Works over SSH with no GUI dependencies
