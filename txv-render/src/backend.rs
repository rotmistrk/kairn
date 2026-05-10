//! CrosstermBackend — implements txv_core::Backend for crossterm terminals.

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor, event as ct_event, execute, queue,
    style::{self, Attribute, SetAttribute},
    terminal,
};
use txv_core::cell::{Attrs, Color, Style};
use txv_core::event::{Event, KeyCode, KeyEvent, KeyMod, MouseAction, MouseButton, MouseEvent};
use txv_core::run::Backend;
use txv_core::surface::Surface;

use crate::color::{downgrade, ColorMode};

/// Crossterm-based terminal backend with dual-buffer diffing.
pub struct CrosstermBackend {
    previous: Surface,
    color_mode: ColorMode,
    force_full: bool,
}

impl CrosstermBackend {
    pub fn new(color_mode: ColorMode) -> Self {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        Self {
            previous: Surface::new(w, h),
            color_mode,
            force_full: true, // First frame always full
        }
    }

    /// Force next flush to emit all cells (no diff).
    pub fn invalidate(&mut self) {
        self.force_full = true;
    }
}

impl Backend for CrosstermBackend {
    fn enter(&mut self) {
        terminal::enable_raw_mode().ok();
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide).ok();
    }

    fn leave(&mut self) {
        execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen).ok();
        terminal::disable_raw_mode().ok();
    }

    fn size(&self) -> (u16, u16) {
        terminal::size().unwrap_or((80, 24))
    }

    fn poll_event(&mut self, timeout: Duration) -> Option<Event> {
        if !ct_event::poll(timeout).unwrap_or(false) {
            return None;
        }
        match ct_event::read() {
            Ok(ct_event::Event::Key(k)) => translate_key(k),
            Ok(ct_event::Event::Resize(w, h)) => Some(Event::Resize(w, h)),
            Ok(ct_event::Event::Mouse(m)) => translate_mouse(m),
            _ => None,
        }
    }

    fn flush(&mut self, surface: &Surface) {
        let w = surface.width();
        let h = surface.height();

        // Resize or force-full: invalidate previous buffer so all cells are emitted
        if self.previous.width() != w || self.previous.height() != h {
            self.previous = Surface::new(w, h);
            self.force_full = true;
        }

        let mut out = io::stdout().lock();
        let mut last_style: Option<Style> = None;

        for y in 0..h {
            let mut run_start: Option<u16> = None;

            for x in 0..w {
                let cell = surface.cell(x, y);

                if !self.force_full {
                    let prev = self.previous.cell(x, y);
                    if cell.ch == prev.ch && cell.style == prev.style {
                        continue;
                    }
                }

                // Need to emit this cell
                if run_start.is_none() || run_start != Some(x) {
                    queue!(out, cursor::MoveTo(x, y)).ok();
                }

                let style = apply_color_mode(cell.style, self.color_mode);
                if last_style.as_ref() != Some(&style) {
                    emit_style(&mut out, &style);
                    last_style = Some(style);
                }

                queue!(out, style::Print(cell.ch)).ok();
                run_start = Some(x + 1);
            }
        }

        queue!(out, SetAttribute(Attribute::Reset)).ok();
        out.flush().ok();

        self.force_full = false;

        // Copy current to previous (full copy, always)
        for y in 0..h {
            for x in 0..w {
                let cell = surface.cell(x, y);
                self.previous.put(x, y, cell.ch, cell.style);
            }
        }
    }
}

fn apply_color_mode(style: Style, mode: ColorMode) -> Style {
    Style {
        fg: downgrade(style.fg, mode),
        bg: downgrade(style.bg, mode),
        attrs: style.attrs,
    }
}

fn emit_style(out: &mut impl Write, style: &Style) {
    queue!(out, SetAttribute(Attribute::Reset)).ok();
    emit_fg(out, style.fg);
    emit_bg(out, style.bg);
    emit_attrs(out, style.attrs);
}

fn emit_fg(out: &mut impl Write, color: Color) {
    let ct_color = to_crossterm_color(color);
    queue!(out, style::SetForegroundColor(ct_color)).ok();
}

fn emit_bg(out: &mut impl Write, color: Color) {
    let ct_color = to_crossterm_color(color);
    queue!(out, style::SetBackgroundColor(ct_color)).ok();
}

fn emit_attrs(out: &mut impl Write, attrs: Attrs) {
    if attrs.bold {
        queue!(out, SetAttribute(Attribute::Bold)).ok();
    }
    if attrs.dim {
        queue!(out, SetAttribute(Attribute::Dim)).ok();
    }
    if attrs.italic {
        queue!(out, SetAttribute(Attribute::Italic)).ok();
    }
    if attrs.underline {
        queue!(out, SetAttribute(Attribute::Underlined)).ok();
    }
    if attrs.reverse {
        queue!(out, SetAttribute(Attribute::Reverse)).ok();
    }
}

fn to_crossterm_color(color: Color) -> style::Color {
    match color {
        Color::Reset => style::Color::Reset,
        Color::Ansi(n) => style::Color::AnsiValue(n),
        Color::Palette(n) => style::Color::AnsiValue(n),
        Color::Rgb(r, g, b) => style::Color::Rgb { r, g, b },
    }
}

fn translate_key(key: ct_event::KeyEvent) -> Option<Event> {
    // Ignore key release events
    if key.kind == ct_event::KeyEventKind::Release {
        return None;
    }

    let modifiers = KeyMod {
        ctrl: key.modifiers.contains(ct_event::KeyModifiers::CONTROL),
        alt: key.modifiers.contains(ct_event::KeyModifiers::ALT),
        shift: key.modifiers.contains(ct_event::KeyModifiers::SHIFT),
    };

    let code = match key.code {
        ct_event::KeyCode::Char(c) => KeyCode::Char(c),
        ct_event::KeyCode::F(n) => KeyCode::F(n),
        ct_event::KeyCode::Enter => KeyCode::Enter,
        ct_event::KeyCode::Esc => KeyCode::Esc,
        ct_event::KeyCode::Tab => KeyCode::Tab,
        ct_event::KeyCode::BackTab => KeyCode::BackTab,
        ct_event::KeyCode::Backspace => KeyCode::Backspace,
        ct_event::KeyCode::Delete => KeyCode::Delete,
        ct_event::KeyCode::Left => KeyCode::Left,
        ct_event::KeyCode::Right => KeyCode::Right,
        ct_event::KeyCode::Up => KeyCode::Up,
        ct_event::KeyCode::Down => KeyCode::Down,
        ct_event::KeyCode::Home => KeyCode::Home,
        ct_event::KeyCode::End => KeyCode::End,
        ct_event::KeyCode::PageUp => KeyCode::PageUp,
        ct_event::KeyCode::PageDown => KeyCode::PageDown,
        ct_event::KeyCode::Insert => KeyCode::Insert,
        _ => return None,
    };

    Some(Event::Key(KeyEvent { code, modifiers }))
}

fn translate_mouse(m: ct_event::MouseEvent) -> Option<Event> {
    let modifiers = KeyMod {
        ctrl: m.modifiers.contains(ct_event::KeyModifiers::CONTROL),
        alt: m.modifiers.contains(ct_event::KeyModifiers::ALT),
        shift: m.modifiers.contains(ct_event::KeyModifiers::SHIFT),
    };

    let action = match m.kind {
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Left) => MouseAction::Press(MouseButton::Left),
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Right) => MouseAction::Press(MouseButton::Right),
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Middle) => MouseAction::Press(MouseButton::Middle),
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Left) => MouseAction::Release(MouseButton::Left),
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Right) => MouseAction::Release(MouseButton::Right),
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Middle) => MouseAction::Release(MouseButton::Middle),
        ct_event::MouseEventKind::Moved | ct_event::MouseEventKind::Drag(_) => MouseAction::Move,
        ct_event::MouseEventKind::ScrollUp => MouseAction::ScrollUp,
        ct_event::MouseEventKind::ScrollDown => MouseAction::ScrollDown,
        _ => return None,
    };

    Some(Event::Mouse(MouseEvent {
        x: m.column,
        y: m.row,
        action,
        modifiers,
    }))
}

/// Compute which cells changed between current surface and previous buffer.
/// Returns list of (x, y) positions that differ.
pub fn diff_cells(current: &Surface, previous: &Surface) -> Vec<(u16, u16)> {
    let mut changed = Vec::new();
    let w = current.width().min(previous.width());
    let h = current.height().min(previous.height());
    for y in 0..h {
        for x in 0..w {
            let c = current.cell(x, y);
            let p = previous.cell(x, y);
            if c.ch != p.ch || c.style != p.style {
                changed.push((x, y));
            }
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use txv_core::cell::Style;
    use txv_core::surface::Surface;

    #[test]
    fn diff_detects_all_changed_cells() {
        let mut s1 = Surface::new(10, 5);
        s1.print(0, 0, "hello", Style::default());
        s1.print(0, 1, "world", Style::default());

        // Simulate: previous = copy of s1
        let mut prev = Surface::new(10, 5);
        for y in 0..5 {
            for x in 0..10 {
                let c = s1.cell(x, y);
                prev.put(x, y, c.ch, c.style);
            }
        }

        // Draw different content
        let mut s2 = Surface::new(10, 5);
        s2.fill(' ', Style::default());
        s2.print(0, 0, "HELLO", Style::default());
        s2.print(0, 1, "WORLD", Style::default());

        let changed = diff_cells(&s2, &prev);

        // All 10 chars on rows 0-1 changed (lowercase → uppercase)
        for x in 0..5u16 {
            assert!(changed.contains(&(x, 0)), "({},0) should be changed", x);
            assert!(changed.contains(&(x, 1)), "({},1) should be changed", x);
        }
    }

    #[test]
    fn diff_detects_style_changes() {
        let mut s1 = Surface::new(10, 1);
        s1.print(0, 0, "test", Style::default());

        let mut prev = Surface::new(10, 1);
        for x in 0..10 {
            let c = s1.cell(x, 0);
            prev.put(x, 0, c.ch, c.style);
        }

        // Same chars but different style
        let mut s2 = Surface::new(10, 1);
        let bold = Style { attrs: Attrs { bold: true, ..Attrs::default() }, ..Style::default() };
        s2.print(0, 0, "test", bold);

        let changed = diff_cells(&s2, &prev);
        assert_eq!(changed.len(), 4, "4 cells changed style");
    }

    #[test]
    fn previous_buffer_updated_after_flush_simulation() {
        // Simulate the flush copy logic
        let mut prev = Surface::new(10, 3);

        // Frame 1: draw "AAAA"
        let mut frame1 = Surface::new(10, 3);
        frame1.fill(' ', Style::default());
        frame1.print(0, 0, "AAAA", Style::default());

        // Copy frame1 → prev (simulating end of flush)
        for y in 0..3 {
            for x in 0..10 {
                let c = frame1.cell(x, y);
                prev.put(x, y, c.ch, c.style);
            }
        }

        // Frame 2: draw "BB" (shorter)
        let mut frame2 = Surface::new(10, 3);
        frame2.fill(' ', Style::default());
        frame2.print(0, 0, "BB", Style::default());

        let changed = diff_cells(&frame2, &prev);

        // Cells 0,1 changed (A→B), cells 2,3 changed (A→space)
        assert!(changed.contains(&(0, 0)), "cell 0 should change A→B");
        assert!(changed.contains(&(1, 0)), "cell 1 should change A→B");
        assert!(changed.contains(&(2, 0)), "cell 2 should change A→space");
        assert!(changed.contains(&(3, 0)), "cell 3 should change A→space");
    }
}
