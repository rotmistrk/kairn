# Welcome Screen: Runtime Dependency Checklist

## Goal

The welcome screen should show which optional runtime tools are available and which
are missing, with version info and installation hints. Helps users understand what
features are available without trial-and-error.

## Dependencies to check

| Tool | Purpose | Install hint |
|------|---------|--------------|
| kiro-cli | AI chat tabs | `cargo install kiro-cli` or download from releases |
| rust-analyzer | Rust LSP | `rustup component add rust-analyzer` |
| gopls | Go LSP | `go install golang.org/x/tools/gopls@latest` |
| typescript-language-server | TS/JS LSP | `npm i -g typescript-language-server typescript` |
| clangd | C/C++ LSP | comes with LLVM / `brew install llvm` |
| jdtls | Java LSP | Eclipse JDT.LS download |
| pyright-langserver | Python LSP | `npm i -g pyright` |
| pbcopy/xclip | Clipboard | macOS built-in / `apt install xclip` |

## Design

### Detection (lazy, cached)

```rust
struct ToolStatus {
    name: &'static str,
    found: bool,
    version: Option<String>,  // populated lazily
    install_hint: &'static str,
}
```

Detection: `which <tool>` or `Command::new(tool).arg("--version").output()`.
Run on first draw, cache results. Version populated async (don't block draw).

### Display

Below the logo, show a checklist:

```
  ✓ rust-analyzer 2024-01-15
  ✓ kiro-cli 0.3.2
  ✗ gopls — go install golang.org/x/tools/gopls@latest
  ✗ clangd — brew install llvm
  ─ typescript-language-server (not needed for this project)
```

- ✓ green: found (show version when available)
- ✗ dim/red: not found (show install hint)
- ─ gray: not relevant for current project (no matching files in tree)

### Project relevance

Scan the project root for file extensions to determine which LSP servers are
relevant. Don't show "gopls missing" in a pure Rust project.

### Implementation

1. Add `ToolChecker` struct with lazy detection
2. On first `draw()`, spawn checks (non-blocking for version queries)
3. Render checklist below existing welcome content
4. Use glyphs for checkmarks (✓/✗ in unicode, [x]/[ ] in ascii)

## Estimated LOE

1.5-2 hours
