//! Tests for InputLine overflow indicators and cursor-not-on-overflow logic.

mod helpers;

use txv_core::event::{Event, KeyCode, KeyEvent, KeyMod};
use txv_core::geometry::Rect;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::View;
use txv_widgets::input_line::InputLine;

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyMod::default()))
}

#[test]
fn cursor_not_on_right_overflow_position() {
    let mut input = InputLine::new();
    input.set_text("abcdefghij");
    input.set_bounds(Rect::new(0, 0, 5, 1));
    input.select();
    input.handle(&Event::Key(KeyEvent::new(KeyCode::Home, KeyMod::default())));
    for _ in 0..4 {
        input.handle(&key(KeyCode::Right));
    }
    let cr = input.cursor().expect("cursor request");
    assert!(
        cr.x() < 4,
        "cursor should not be on the right-overflow position, got x={}",
        cr.x()
    );
}

#[test]
fn cursor_not_on_left_overflow_position() {
    let mut input = InputLine::new();
    input.set_text("abcdef");
    input.set_bounds(Rect::new(0, 0, 2, 1));
    input.select();
    input.handle(&Event::Key(KeyEvent::new(KeyCode::End, KeyMod::default())));
    for _ in 0..4 {
        input.handle(&key(KeyCode::Left));
    }
    let cr = input.cursor().expect("cursor request");
    assert!(
        cr.x() > 0,
        "cursor should not be on the left-overflow position, got x={}",
        cr.x()
    );
}
