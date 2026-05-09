use super::*;
use crate::event::{Event, KeyCode, KeyEvent, KeyMod};
use crate::surface::Surface;
use crate::view::{EventQueue, HandleResult, ViewState};

struct DummyView { state: ViewState }
impl DummyView {
    fn new(focusable: bool) -> Self {
        Self { state: ViewState::new(ViewOptions { focusable, ..ViewOptions::default() }) }
    }
}
impl View for DummyView {
    crate::delegate_view_state!(state);
    fn draw(&self, _surface: &mut Surface) {}
    fn handle(&mut self, _event: &Event, _queue: &mut EventQueue) -> HandleResult { HandleResult::Ignored }
}

#[test]
fn focus_next_skips_unfocusable() {
    let mut g = GroupState::default();
    g.insert(Box::new(DummyView::new(true)));
    g.insert(Box::new(DummyView::new(false)));
    g.insert(Box::new(DummyView::new(true)));
    g.children[0].select();
    g.focus_next();
    assert_eq!(g.focused, 2);
}

#[test]
fn focus_prev_wraps() {
    let mut g = GroupState::default();
    g.insert(Box::new(DummyView::new(true)));
    g.insert(Box::new(DummyView::new(true)));
    g.insert(Box::new(DummyView::new(true)));
    g.children[0].select();
    g.focus_prev();
    assert_eq!(g.focused, 2);
}

#[test]
fn three_phase_dispatch() {
    struct PreView { state: ViewState }
    impl View for PreView {
        crate::delegate_view_state!(state);
        fn draw(&self, _s: &mut Surface) {}
        fn handle(&mut self, _event: &Event, _queue: &mut EventQueue) -> HandleResult { HandleResult::Consumed }
    }

    let mut g = GroupState::default();
    g.insert(Box::new(PreView {
        state: ViewState::new(ViewOptions { preprocess: true, focusable: false, ..ViewOptions::default() }),
    }));
    g.insert(Box::new(DummyView::new(true)));
    g.focused = 1;

    let ev = Event::Key(KeyEvent { code: KeyCode::Char('x'), modifiers: KeyMod::default() });
    let mut queue = EventQueue::new();
    let result = g.dispatch(&ev, &mut queue);
    assert_eq!(result, HandleResult::Consumed);
}
