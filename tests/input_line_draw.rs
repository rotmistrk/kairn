//! Tests for InputLine draw correctness: colors, selection, overflow, no extra chars.

mod helpers;

use txv_core::cell::Style;
use txv_core::event::{Event, KeyCode, KeyEvent, KeyMod};
use txv_core::geometry::Rect;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::View;
use txv_widgets::input_line::InputLine;

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyMod::default(),
    })
}

fn shift_key(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    })
}

/// Draw InputLine and return buffer cells as (char, style) pairs.
fn draw_cells(input: &mut InputLine, width: u16) -> Vec<(char, Style)> {
    input.set_bounds(Rect::new(0, 0, width, 1));
    // Don't call select() — it triggers select_all(). Just draw.
    input.draw();
    let buf = input.buffer();
    (0..width)
        .map(|x| {
            let cell = buf.cell(x, 0);
            (cell.ch, cell.style)
        })
        .collect()
}

#[test]
fn no_extra_chars_after_text() {
    let mut input = InputLine::new();
    input.set_text("hello");
    let cells = draw_cells(&mut input, 20);
    assert_eq!(cells[0].0, 'h');
    assert_eq!(cells[4].0, 'o');
    // Cell 5 is cursor (space), cells 6..20 must be spaces
    for i in 6..20 {
        assert_eq!(cells[i].0, ' ', "cell {i} should be space, got '{}'", cells[i].0);
    }
}

#[test]
fn background_fills_entire_width() {
    let mut input = InputLine::new();
    input.set_text("ab");
    let cells = draw_cells(&mut input, 10);
    let text_style = palette().style(StyleId::Text);
    // All non-cursor cells should have Text bg
    for i in 0..10 {
        if i == 2 {
            continue; // cursor position
        }
        assert_eq!(
            cells[i].1.bg, text_style.bg,
            "cell {i} bg should be Text bg ({:?}), got {:?}",
            text_style.bg, cells[i].1.bg
        );
    }
}

#[test]
fn cursor_at_end_of_text() {
    use txv_core::cursor::{CursorRequest, CursorShape};
    use txv_core::geometry::Rect;
    let mut input = InputLine::new();
    input.set_text("abc");
    assert_eq!(input.cursor_pos(), 3);
    input.set_bounds(Rect::new(0, 0, 10, 1));
    input.select(); // sets focused + select_all
                    // Deselect text but keep focused — press End to clear selection
    input.handle(&Event::Key(KeyEvent {
        code: KeyCode::End,
        modifiers: KeyMod::default(),
    }));
    // Hardware cursor should be at x=3 with Bar shape
    let cr = input.cursor().expect("should return CursorRequest");
    assert_eq!(cr.x, 3, "hardware cursor x should be at end of text");
    assert_eq!(cr.shape, CursorShape::Bar, "should use Bar shape");
}

#[test]
fn selection_via_shift_right() {
    let mut input = InputLine::new();
    input.set_text("abcdef");
    // Move cursor to start
    input.handle(&key(KeyCode::Home));
    // Shift+Right 3 times → select chars 0..3
    input.handle(&shift_key(KeyCode::Right));
    input.handle(&shift_key(KeyCode::Right));
    input.handle(&shift_key(KeyCode::Right));
    let cells = draw_cells(&mut input, 10);
    let sel_style = palette().style(StyleId::EditSelection);
    let text_style = palette().style(StyleId::Text);
    assert_eq!(cells[0].1, sel_style, "cell 0 should be selected");
    assert_eq!(cells[1].1, sel_style, "cell 1 should be selected");
    assert_eq!(cells[2].1, sel_style, "cell 2 should be selected");
    assert_eq!(cells[3].1, text_style, "cell 3 should NOT be selected");
    assert_eq!(cells[4].1, text_style, "cell 4 should NOT be selected");
}

#[test]
fn select_all_highlights_entire_text() {
    let mut input = InputLine::new();
    input.set_text("hello");
    input.select_all();
    let cells = draw_cells(&mut input, 10);
    let sel_style = palette().style(StyleId::EditSelection);
    let text_style = palette().style(StyleId::Text);
    for i in 0..5 {
        assert_eq!(cells[i].1, sel_style, "cell {i} should be EditSelection");
    }
    for i in 5..10 {
        assert_eq!(cells[i].1, text_style, "cell {i} past text should be Text");
    }
}

#[test]
fn overflow_right_indicator() {
    let mut input = InputLine::new();
    input.set_text("abcdefghij"); // 10 chars
                                  // Move cursor to start so text overflows right
    input.handle(&key(KeyCode::Home));
    let cells = draw_cells(&mut input, 5);
    assert_eq!(cells[4].0, '…', "rightmost cell should be overflow '…'");
    let ov_fg = palette().style(StyleId::OverflowIndicator).fg;
    assert_eq!(cells[4].1.fg, ov_fg, "overflow should use OverflowIndicator fg");
}

#[test]
fn overflow_left_indicator() {
    let mut input = InputLine::new();
    input.set_text("abcdefghij"); // 10 chars, cursor at end (pos 10)
    let cells = draw_cells(&mut input, 5);
    assert_eq!(cells[0].0, '…', "leftmost cell should be overflow '…'");
    let ov_fg = palette().style(StyleId::OverflowIndicator).fg;
    assert_eq!(cells[0].1.fg, ov_fg, "left overflow should use OverflowIndicator fg");
}

#[test]
fn overflow_both_indicators() {
    let mut input = InputLine::new();
    input.set_text("abcdefghijklmno"); // 15 chars
                                       // Move cursor to middle
    input.handle(&key(KeyCode::Home));
    for _ in 0..7 {
        input.handle(&key(KeyCode::Right));
    }
    let cells = draw_cells(&mut input, 5);
    assert_eq!(cells[0].0, '…', "left overflow");
    assert_eq!(cells[4].0, '…', "right overflow");
}

#[test]
fn no_overflow_when_text_fits() {
    let mut input = InputLine::new();
    input.set_text("abc");
    let cells = draw_cells(&mut input, 10);
    for i in 0..10 {
        assert_ne!(cells[i].0, '…', "cell {i} should not be overflow indicator");
    }
}

#[test]
fn selection_backwards_via_shift_left() {
    let mut input = InputLine::new();
    input.set_text("abcdef");
    // Cursor at end (6). Shift+Left 3 times → select 3..6
    input.handle(&shift_key(KeyCode::Left));
    input.handle(&shift_key(KeyCode::Left));
    input.handle(&shift_key(KeyCode::Left));
    let cells = draw_cells(&mut input, 10);
    let sel_style = palette().style(StyleId::EditSelection);
    let text_style = palette().style(StyleId::Text);
    assert_eq!(cells[0].1, text_style, "cell 0 before selection");
    assert_eq!(cells[1].1, text_style, "cell 1 before selection");
    assert_eq!(cells[2].1, text_style, "cell 2 before selection");
    assert_eq!(cells[3].1, sel_style, "cell 3 in selection");
    assert_eq!(cells[4].1, sel_style, "cell 4 in selection");
    assert_eq!(cells[5].1, sel_style, "cell 5 in selection");
}

#[test]
fn typing_replaces_selection_no_extra_chars() {
    let mut input = InputLine::new();
    input.set_text("hello");
    input.select_all(); // all selected, cursor at 5
                        // Type 'x' — should replace entire selection
    input.handle(&key(KeyCode::Char('x')));
    assert_eq!(input.text(), "x");
    let cells = draw_cells(&mut input, 10);
    assert_eq!(cells[0].0, 'x');
    // Cell 1 is cursor, rest spaces
    for i in 2..10 {
        assert_eq!(cells[i].0, ' ', "cell {i} should be space after replace");
    }
}

#[test]
fn view_select_triggers_select_all() {
    let mut input = InputLine::new();
    input.set_text("hello");
    input.set_bounds(Rect::new(0, 0, 10, 1));
    input.select(); // View::select — should select all text
    input.draw();
    let buf = input.buffer();
    let sel_style = palette().style(StyleId::EditSelection);
    for i in 0..5u16 {
        assert_eq!(
            buf.cell(i, 0).style,
            sel_style,
            "cell {i} should be selected after View::select()"
        );
    }
}

#[test]
fn paste_multiline_takes_first_line_only() {
    use txv_core::event::Event;
    use txv_widgets::input_line::CM_CLIPBOARD_PASTE;

    let mut input = InputLine::new();
    input.set_text("");
    input.set_bounds(Rect::new(0, 0, 40, 1));
    input.select();
    let paste = Event::Command {
        broadcast: false,
        id: CM_CLIPBOARD_PASTE,
        data: Some(Box::new("first\nsecond\nthird".to_string())),
    };
    input.handle(&paste);
    let cells = draw_cells(&mut input, 40);
    let text: String = cells.iter().map(|(ch, _)| ch).collect();
    let text = text.trim_end();
    assert_eq!(text, "first", "should only contain first line, got: '{text}'");
}
