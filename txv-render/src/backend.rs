//! CrosstermBackend — implements txv_core::Backend for crossterm terminals.

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor, event as ct_event, execute, queue,
    style::{self, Attribute, SetAttribute},
    terminal,
};
use txv_core::cell::{Attrs, Color, Style};
use txv_core::event::{
    Event, KeyCode, KeyEvent, KeyMod, MouseAction, MouseButton, MouseEvent,
};
use txv_core::run::Backend;
use txv_core::surface::Surface;

use crate::color::{downgrade, ColorMode};

/// Crossterm-based terminal backend with dual-buffer diffing.
pub struct CrosstermBackend {
    previous: Surface,
    color_mode: ColorMode,
}

impl CrosstermBackend {
    pub fn new(color_mode: ColorMode) -> Self {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        Self {
            previous: Surface::new(w, h),
            color_mode,
        }
    }
}

impl Backend for CrosstermBackend {
    fn enter(&mut self) {
        terminal::enable_raw_mode().ok();
        execute!(
            io::stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide
        )
        .ok();
    }

    fn leave(&mut self) {
        execute!(
            io::stdout(),
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
            _ => None,
        }
    }

    fn flush(&mut self, surface: &Surface) {
        let w = surface.width();
        let h = surface.height();

        // Resize previous buffer if terminal size changed
        if self.previous.width() != w || self.previous.height() != h {
            self.previous = Surface::new(w, h);
        }

        let mut out = io::stdout().lock();
        let mut last_style: Option<Style> = None;

        for y in 0..h {
            let mut run_start: Option<u16> = None;

            for x in 0..w {
                let cell = surface.cell(x, y);
                let prev = self.previous.cell(x, y);

                if cell.ch == prev.ch && cell.style == prev.style {
                    continue;
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

        // Copy current to previous
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
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Left) => {
            MouseAction::Press(MouseButton::Left)
        }
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Right) => {
            MouseAction::Press(MouseButton::Right)
        }
        ct_event::MouseEventKind::Down(ct_event::MouseButton::Middle) => {
            MouseAction::Press(MouseButton::Middle)
        }
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Left) => {
            MouseAction::Release(MouseButton::Left)
        }
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Right) => {
            MouseAction::Release(MouseButton::Right)
        }
        ct_event::MouseEventKind::Up(ct_event::MouseButton::Middle) => {
            MouseAction::Release(MouseButton::Middle)
        }
        ct_event::MouseEventKind::Moved | ct_event::MouseEventKind::Drag(_) => {
            MouseAction::Move
        }
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
