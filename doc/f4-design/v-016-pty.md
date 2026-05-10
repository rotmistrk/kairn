# v-016: PTY Terminal Widget

## Overview

A reusable terminal widget in txv-widgets that spawns a real PTY process,
renders its output via TermBuf (txv-render), and forwards keyboard input.

## Architecture

```
txv-render/src/termbuf/   — VTE terminal emulator (exists, parses ANSI → cell grid)
txv-widgets/src/pty_terminal.rs — PtyTerminal View (new)
  - Owns: TermBuf + PtySession
  - On Tick: polls PTY output, feeds to TermBuf
  - On Key: translates to bytes, writes to PTY
  - On draw: renders TermBuf cells to Surface
  - On resize: resizes PTY + TermBuf
kairn — just opens PtyTerminal tabs via commands
```

## PtyTerminal (txv-widgets)

```rust
pub struct PtyTerminal {
    state: ViewState,
    termbuf: TermBuf,
    session: Option<PtySession>,
    title: String,
}

impl PtyTerminal {
    pub fn spawn_shell(cols: u16, rows: u16) -> Result<Self>;
    pub fn spawn_command(cmd: &str, args: &[&str], cwd: &Path, cols: u16, rows: u16) -> Result<Self>;
}
```

## PtySession (internal to txv-widgets)

```rust
struct PtySession {
    writer: Box<dyn Write + Send>,
    rx: mpsc::Receiver<Vec<u8>>,
    master: Box<dyn MasterPty + Send>,  // for resize
}

impl PtySession {
    fn spawn(cmd: &str, args: &[&str], cwd: &Path, cols: u16, rows: u16) -> Result<Self>;
    fn poll(&self) -> Option<Vec<u8>>;
    fn write(&mut self, data: &[u8]);
    fn resize(&self, cols: u16, rows: u16);
}
```

## Event Flow

```
Tick → PtyTerminal.handle()
  → session.poll() → if data: termbuf.process(data), mark dirty
  → drain termbuf responses → session.write(responses)

Key → PtyTerminal.handle()
  → key_to_bytes(key) → session.write(bytes)

Resize → PtyTerminal.set_bounds()
  → session.resize(new_cols, new_rows)
  → termbuf.resize(new_cols, new_rows)

Draw → PtyTerminal.draw()
  → termbuf.render_to(surface, bounds)
```

## Key Translation

Standard xterm key encoding (from old terminal_panel.rs):
- Chars → UTF-8 bytes
- Ctrl-C → 0x03, Ctrl-D → 0x04, etc.
- Arrows → ESC [ A/B/C/D
- F1-F12 → standard escape sequences
- Enter → CR, Backspace → DEL (0x7f), Tab → 0x09

## Dependencies

txv-widgets/Cargo.toml:
```toml
[dependencies]
portable-pty = "0.8"
txv-render = { path = "../txv-render" }
```

## TermBuf Integration

TermBuf already has:
- process(data: &[u8]) — feed VTE data
- render_to(surface) — draw cells to Surface
- resize(cols, rows)

Need to add/verify:
- responses/drain_responses — DA1, CPR replies sent back to PTY
- Cursor rendering (reverse attr at cursor position)

## Kairn Integration

Replace current placeholder TerminalView with:
```rust
// In handler.rs CM_NEW_SHELL:
let term = PtyTerminal::spawn_shell(cols, rows)?;
desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));

// For kiro:
let term = PtyTerminal::spawn_command("kiro", &["chat"], &root_dir, cols, rows)?;
desktop.insert_tab(SlotId::Right, "Kiro", Box::new(term));
```

## Testing

- Unit test: spawn `echo hello && exit`, poll until output contains "hello"
- Unit test: spawn `cat`, write "test\n", poll until output contains "test"
- Unit test: resize doesn't panic
- Integration: TestHarness opens shell tab, verifies it renders something

## File Structure

```
txv-widgets/src/pty_terminal.rs  — PtyTerminal View impl
txv-widgets/src/pty_session.rs   — PtySession (spawn, poll, write, resize)
txv-widgets/src/key_encode.rs    — key_to_bytes translation
```
