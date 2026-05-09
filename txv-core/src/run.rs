//! Backend trait, run loop, exec_view (modal), and MockBackend for tests.

use std::time::Duration;

use crate::cell::Style;
use crate::commands::{CM_CANCEL, CM_CLOSE, CM_OK, CM_QUIT};
use crate::event::{CommandId, Event, KeyCode, KeyEvent, KeyMod};
use crate::surface::Surface;
use crate::view::{EventQueue, View};

/// Backend trait — implemented by terminal renderers.
pub trait Backend: Send {
    /// Poll for input events. Returns None on timeout.
    fn poll_event(&mut self, timeout: Duration) -> Option<Event>;
    /// Get current terminal/window size.
    fn size(&self) -> (u16, u16);
    /// Flush a surface to the display.
    fn flush(&mut self, surface: &Surface);
    /// Enter TUI mode.
    fn enter(&mut self);
    /// Leave TUI mode.
    fn leave(&mut self);
}

/// Run the main event loop. Returns when CM_QUIT is received.
pub fn run(root: &mut dyn View, backend: &mut dyn Backend) {
    backend.enter();
    let mut queue = EventQueue::new();
    let (w, h) = backend.size();
    let mut surface = Surface::new(w, h);

    loop {
        if root.needs_redraw() {
            surface.fill(' ', Style::default());
            root.draw(&mut surface);
            root.mark_redrawn();
            backend.flush(&surface);
        }

        if let Some(event) = backend.poll_event(Duration::from_millis(50)) {
            if let Event::Resize(nw, nh) = &event {
                surface = Surface::new(*nw, *nh);
            }
            root.handle(&event, &mut queue);
        } else {
            root.handle(&Event::Tick, &mut queue);
        }

        let events = queue.drain();
        for ev in events {
            if let Event::Command { id, .. } = &ev {
                if *id == CM_QUIT {
                    backend.leave();
                    return;
                }
            }
            root.handle(&ev, &mut queue);
        }
    }
}

/// Modal nested event loop. Key/Mouse → modal only. Tick/Resize/Command → full tree.
/// Returns the closing command (CM_CLOSE, CM_OK, or CM_CANCEL).
pub fn exec_view(
    root: &mut dyn View,
    modal: &mut dyn View,
    backend: &mut dyn Backend,
) -> CommandId {
    let mut queue = EventQueue::new();

    loop {
        let (w, h) = backend.size();
        let mut surface = Surface::new(w, h);
        root.draw(&mut surface);
        modal.draw(&mut surface);
        backend.flush(&surface);

        match backend.poll_event(Duration::from_millis(50)) {
            Some(Event::Key(ref k)) => {
                let ev = Event::Key(k.clone());
                modal.handle(&ev, &mut queue);
            }
            Some(Event::Mouse(m)) => {
                modal.handle(&Event::Mouse(m), &mut queue);
            }
            Some(Event::Resize(nw, nh)) => {
                let ev = Event::Resize(nw, nh);
                root.handle(&ev, &mut queue);
                modal.handle(&ev, &mut queue);
            }
            Some(Event::Tick) | None => {
                root.handle(&Event::Tick, &mut queue);
                modal.handle(&Event::Tick, &mut queue);
            }
            Some(ev @ Event::Command { .. }) => {
                root.handle(&ev, &mut queue);
            }
        }

        let events = queue.drain();
        for ev in events {
            if let Event::Command { id, .. } = &ev {
                if matches!(*id, CM_CLOSE | CM_OK | CM_CANCEL) {
                    return *id;
                }
            }
            root.handle(&ev, &mut queue);
        }
    }
}

/// Run N event-loop cycles with a MockBackend (for testing).
/// Processes all injected events, dispatches queued commands, draws.
pub fn run_cycles(root: &mut dyn View, backend: &mut MockBackend, n: usize) {
    let mut queue = EventQueue::new();
    let (w, h) = backend.size();
    let mut surface = Surface::new(w, h);

    for _ in 0..n {
        while let Some(event) = backend.poll_event(Duration::ZERO) {
            if let Event::Resize(nw, nh) = &event {
                surface = Surface::new(*nw, *nh);
            }
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
        Self {
            width,
            height,
            events: Vec::new(),
            last_surface: None,
        }
    }

    /// Inject an event to be returned by the next poll_event call.
    pub fn inject(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Inject a key event.
    pub fn inject_key(&mut self, code: KeyCode, modifiers: KeyMod) {
        self.inject(Event::Key(KeyEvent { code, modifiers }));
    }

    /// Inject a sequence of characters as key events.
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

    /// Get the last flushed surface.
    pub fn surface(&self) -> Option<&Surface> {
        self.last_surface.as_ref()
    }

    /// Get the full screen as a string (rows joined by newline, trailing spaces trimmed).
    pub fn screen_text(&self) -> String {
        let Some(ref s) = self.last_surface else {
            return String::new();
        };
        let mut rows = Vec::new();
        for y in 0..s.height() {
            rows.push(self.row(y));
        }
        rows.join("\n")
    }

    /// Check if any row contains the given text.
    pub fn contains(&self, text: &str) -> bool {
        let Some(ref s) = self.last_surface else {
            return false;
        };
        for y in 0..s.height() {
            if self.row(y).contains(text) {
                return true;
            }
        }
        false
    }

    /// Get text of a specific row (trailing spaces trimmed).
    pub fn row(&self, y: u16) -> String {
        let Some(ref s) = self.last_surface else {
            return String::new();
        };
        if y >= s.height() {
            return String::new();
        }
        let mut row = String::new();
        for x in 0..s.width() {
            row.push(s.cell(x, y).ch);
        }
        row.trim_end().to_string()
    }
}

impl Backend for MockBackend {
    fn poll_event(&mut self, _timeout: Duration) -> Option<Event> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }

    fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{KeyCode, KeyEvent, KeyMod};
    use crate::geometry::Rect;
    use crate::view::{HandleResult, ViewState};

    struct QuitView {
        state: ViewState,
    }

    impl QuitView {
        fn new() -> Self {
            Self {
                state: ViewState::default(),
            }
        }
    }

    impl View for QuitView {
        crate::delegate_view_state!(state);

        fn draw(&self, surface: &mut Surface) {
            surface.put(0, 0, 'Q', Style::default());
        }

        fn handle(
            &mut self,
            event: &Event,
            queue: &mut EventQueue,
        ) -> HandleResult {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) = event
            {
                queue.put_command(CM_QUIT, None);
                return HandleResult::Consumed;
            }
            HandleResult::Ignored
        }
    }

    #[test]
    fn run_quits_on_cm_quit() {
        let mut view = QuitView::new();
        view.set_bounds(Rect::new(0, 0, 80, 24));
        let mut backend = MockBackend::new(80, 24);
        backend.inject(Event::Key(KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyMod::default(),
        }));
        run(&mut view, &mut backend);
        let s = backend.surface().expect("surface should be flushed");
        assert_eq!(s.cell(0, 0).ch, 'Q');
    }

    #[test]
    fn mock_backend_inject_and_poll() {
        let mut b = MockBackend::new(80, 24);
        assert!(b.poll_event(Duration::from_millis(0)).is_none());
        b.inject(Event::Tick);
        assert!(b.poll_event(Duration::from_millis(0)).is_some());
    }
}
