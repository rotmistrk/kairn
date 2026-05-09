//! MockBackend for testing without a terminal.

use std::time::Duration;

use crate::cell::Style;
use crate::commands::CM_QUIT;
use crate::event::{Event, KeyCode, KeyMod};
use crate::surface::Surface;
use crate::view::{EventQueue, View};

use super::Backend;

/// Run N event-loop cycles with a MockBackend (for testing).
pub fn run_cycles(root: &mut dyn View, backend: &mut MockBackend, n: usize) {
    let mut queue = EventQueue::new();
    let (w, h) = backend.size();
    let mut surface = Surface::new(w, h);

    for _ in 0..n {
        while let Some(event) = backend.poll_event(Duration::ZERO) {
            if let Event::Resize(nw, nh) = &event { surface = Surface::new(*nw, *nh); }
            root.handle(&event, &mut queue);

            let events = queue.drain();
            for ev in events {
                if let Event::Command { id, .. } = &ev {
                    if *id == CM_QUIT {
                        surface.fill(' ', Style::default());
                        root.draw(&mut surface);
                        backend.flush(&surface);
                        return;
                    }
                }
                root.handle(&ev, &mut queue);
            }
        }

        surface.fill(' ', Style::default());
        root.draw(&mut surface);
        root.mark_redrawn();
        backend.flush(&surface);
    }
}

/// Mock backend for testing without a terminal.
pub struct MockBackend {
    width: u16,
    height: u16,
    events: Vec<Event>,
    last_surface: Option<Surface>,
}

impl MockBackend {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height, events: Vec::new(), last_surface: None }
    }

    pub fn inject(&mut self, event: Event) { self.events.push(event); }

    pub fn inject_key(&mut self, code: KeyCode, modifiers: KeyMod) {
        self.inject(Event::Key(crate::event::KeyEvent { code, modifiers }));
    }

    pub fn inject_str(&mut self, s: &str) {
        for ch in s.chars() {
            match ch {
                '\n' => self.inject_key(KeyCode::Enter, KeyMod::default()),
                '\x1b' => self.inject_key(KeyCode::Esc, KeyMod::default()),
                '\t' => self.inject_key(KeyCode::Tab, KeyMod::default()),
                c => self.inject_key(KeyCode::Char(c), KeyMod::default()),
            }
        }
    }

    pub fn surface(&self) -> Option<&Surface> { self.last_surface.as_ref() }

    pub fn screen_text(&self) -> String {
        let Some(ref s) = self.last_surface else { return String::new() };
        let mut rows = Vec::new();
        for y in 0..s.height() { rows.push(self.row(y)); }
        rows.join("\n")
    }

    pub fn contains(&self, text: &str) -> bool {
        let Some(ref s) = self.last_surface else { return false };
        for y in 0..s.height() { if self.row(y).contains(text) { return true; } }
        false
    }

    pub fn row(&self, y: u16) -> String {
        let Some(ref s) = self.last_surface else { return String::new() };
        if y >= s.height() { return String::new(); }
        let mut row = String::new();
        for x in 0..s.width() { row.push(s.cell(x, y).ch); }
        row.trim_end().to_string()
    }
}

impl Backend for MockBackend {
    fn poll_event(&mut self, _timeout: Duration) -> Option<Event> {
        if self.events.is_empty() { None } else { Some(self.events.remove(0)) }
    }
    fn size(&self) -> (u16, u16) { (self.width, self.height) }
    fn flush(&mut self, surface: &Surface) {
        self.last_surface = Some(Surface::new(surface.width(), surface.height()));
        if let Some(ref mut s) = self.last_surface {
            for y in 0..surface.height() {
                for x in 0..surface.width() {
                    let cell = surface.cell(x, y);
                    s.put(x, y, cell.ch, cell.style);
                }
            }
        }
    }
    fn enter(&mut self) {}
    fn leave(&mut self) {}
}
