//! CrosstermBackend — implements txv_core::Backend for crossterm terminals.

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor, event as ct_event, execute, queue,
    style::{self, Attribute, SetAttribute},
    terminal,
};
use txv_core::cell::{Attrs, Color, Style};
use txv_core::event::Event;
use txv_core::run::Backend;
use txv_core::surface::Surface;

use crate::color::{downgrade, ColorMode};
use crate::event_translate::{translate_key, translate_mouse};

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
        execute!(
            io::stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            ct_event::EnableBracketedPaste
        )
        .ok();
    }

    fn leave(&mut self) {
        execute!(
            io::stdout(),
            ct_event::DisableBracketedPaste,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .ok();
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
            Ok(ct_event::Event::Paste(s)) => Some(Event::Paste(s)),
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
            let mut cursor_x: Option<u16> = None;

            let mut x = 0u16;
            while x < w {
                let cell = surface.cell(x, y);
                let prev = self.previous.cell(x, y);

                // Skip unchanged cells (including continuation cells)
                if !self.force_full && cell.ch == prev.ch && cell.style == prev.style && cell.width == prev.width {
                    x += 1;
                    cursor_x = None;
                    continue;
                }

                // Skip continuation cells that haven't changed
                if cell.width == 0 && prev.width == 0 {
                    x += 1;
                    continue;
                }

                // Need to emit this cell
                if cursor_x != Some(x) {
                    queue!(out, cursor::MoveTo(x, y)).ok();
                }

                let style = apply_color_mode(cell.style, self.color_mode);
                if last_style.as_ref() != Some(&style) {
                    emit_style(&mut out, &style);
                    last_style = Some(style);
                }

                let ch = if cell.width == 0 {
                    ' '
                } else {
                    cell.ch
                };
                queue!(out, style::Print(ch)).ok();
                cursor_x = Some(x + 1);
                x += 1;
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

    fn invalidate(&mut self) {
        self.force_full = true;
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
#[cfg(test)]
#[path = "backend_tests.rs"]
mod tests;
