use super::*;

struct Dummy {
    state: ViewState,
}
impl Dummy {
    fn new() -> Self {
        Self {
            state: ViewState::default(),
        }
    }
}
impl View for Dummy {
    delegate_view_state!(state);
    fn draw(&self, _: &mut Surface) {}
    fn handle(&mut self, _: &Event, _: &mut EventQueue) -> HandleResult {
        HandleResult::Ignored
    }
}

#[test]
fn insert_and_active() {
    let mut tg = TabGroup::new();
    tg.set_bounds(Rect::new(0, 0, 80, 24));
    tg.insert_tab("A", Box::new(Dummy::new()));
    tg.insert_tab("B", Box::new(Dummy::new()));
    assert_eq!(tg.tab_count(), 2);
    // Last inserted tab becomes active
    assert_eq!(tg.active_title(), Some("B"));
}

#[test]
fn set_active_and_cycle() {
    let mut tg = TabGroup::new();
    tg.set_bounds(Rect::new(0, 0, 80, 24));
    tg.insert_tab("A", Box::new(Dummy::new()));
    tg.insert_tab("B", Box::new(Dummy::new()));
    tg.insert_tab("C", Box::new(Dummy::new()));
    // C is already active (last inserted), switch to A
    tg.set_active(0);
    assert_eq!(tg.active_title(), Some("A"));
    tg.tab_next();
    assert_eq!(tg.active_title(), Some("B"));
}

#[test]
fn close_and_focus_by_title() {
    let mut tg = TabGroup::new();
    tg.set_bounds(Rect::new(0, 0, 80, 24));
    tg.insert_tab("X", Box::new(Dummy::new()));
    tg.insert_tab("Y", Box::new(Dummy::new()));
    assert!(tg.close_tab_by_title("X"));
    assert_eq!(tg.tab_count(), 1);
    assert!(tg.focus_tab_by_title("Y"));
    assert!(!tg.focus_tab_by_title("Z"));
}

#[test]
fn set_bounds_propagates() {
    let mut tg = TabGroup::new();
    tg.insert_tab("T", Box::new(Dummy::new()));
    tg.set_bounds(Rect::new(5, 10, 40, 20));
    assert_eq!(tg.group.child(0).unwrap().bounds(), Rect::new(5, 11, 40, 19));
}
