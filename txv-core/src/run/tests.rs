use std::time::Duration;

use super::*;
use crate::cell::Style;
use crate::commands::CM_QUIT;
use crate::event::{Event, KeyCode, KeyEvent, KeyMod};
use crate::geometry::Rect;
use crate::view::{HandleResult, ViewState};

struct QuitView { state: ViewState }
impl QuitView { fn new() -> Self { Self { state: ViewState::default() } } }
impl View for QuitView {
    crate::delegate_view_state!(state);
    fn draw(&self, surface: &mut Surface) { surface.put(0, 0, 'Q', Style::default()); }
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) = event {
            queue.put_command(CM_QUIT, None);
            return HandleResult::Consumed;
        }
        HandleResult::Ignored
    }
}

#[test]
fn run_quits_on_cm_quit() {
    let mut view = QuitView::new();
    view.state.bounds = Rect::new(0, 0, 80, 24);
    let mut backend = MockBackend::new(80, 24);
    backend.inject(Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyMod::default() }));
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
