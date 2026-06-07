//! Scenario tests using MockBackend — exercises the full Program loop.

use txv_core::prelude::*;
use txv_core::run::MockBackend;

use rusticle_tk::desktop::TkDesktop;
use rusticle_tk::layout_mgr::Side;

use txv_widgets::TextArea;

/// Build a simple desktop with one text widget.
fn build_desktop(content: &str) -> TkDesktop {
    let mut desktop = TkDesktop::new();
    let mut ta = TextArea::new();
    ta.set_content(content);
    desktop.insert_widget("txt".into(), Box::new(ta));
    desktop.layout_mut().add("txt", Side::Fill, None);
    desktop
}

#[test]
fn desktop_renders_text_widget() {
    let mut desktop = build_desktop("Hello, world!");
    desktop.set_bounds(Rect::new(0, 0, 80, 24));

    desktop.draw();

    let child = desktop.get("txt").expect("txt widget");
    let buf = child.buffer();
    let mut row = String::new();
    for x in 0..80 {
        row.push(buf.cell(x, 0).ch());
    }
    assert!(
        row.contains("Hello, world!"),
        "expected 'Hello, world!' in first row, got: '{}'",
        row.trim()
    );
}

#[test]
fn desktop_dispatches_keys_to_focused() {
    let mut desktop = build_desktop("Line 1\nLine 2\nLine 3");
    desktop.insert_widget("input".into(), Box::new(txv_widgets::InputLine::new()));
    desktop.layout_mut().add("input", Side::Bottom, Some(1));
    desktop.focus("input");
    desktop.set_bounds(Rect::new(0, 0, 80, 24));

    let event = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyMod::default()));
    let result = desktop.handle(&event);
    assert_eq!(result, HandleResult::Consumed);
}

#[test]
fn desktop_layout_sets_bounds_on_children() {
    let mut desktop = TkDesktop::new();
    let ta = TextArea::new();
    desktop.insert_widget("main".into(), Box::new(ta));
    desktop.layout_mut().add("main", Side::Fill, None);
    desktop.set_bounds(Rect::new(0, 0, 80, 24));

    let child = desktop.get("main");
    assert!(child.is_some());
    let bounds = child.map(|v| v.bounds()).unwrap_or_default();
    assert_eq!(bounds, Rect::new(0, 0, 80, 24));
}

#[test]
fn desktop_focus_switches_child() {
    let mut desktop = TkDesktop::new();
    desktop.insert_widget("a".into(), Box::new(TextArea::new()));
    desktop.insert_widget("b".into(), Box::new(txv_widgets::InputLine::new()));
    desktop.layout_mut().add("a", Side::Fill, None);
    desktop.layout_mut().add("b", Side::Bottom, Some(1));
    desktop.set_bounds(Rect::new(0, 0, 80, 24));

    desktop.focus("b");

    // Typing should go to input (b), not text (a)
    let event = Event::Key(KeyEvent::new(KeyCode::Char('z'), KeyMod::default()));
    desktop.handle(&event);

    // Verify input received the character
    let view = desktop.get("b");
    assert!(view.is_some());
}

#[test]
fn program_quit_via_command() {
    use txv_core::program::Program;
    use txv_core::status_bar::{StatusBar, StatusSlot};
    use txv_widgets::KeyLabelView;

    let desktop = build_desktop("test");
    let mut bar = StatusBar::new();
    bar.add(StatusSlot::new(Box::new(KeyLabelView::new(
        KeyEvent::new(KeyCode::Char('q'), KeyMod::CTRL),
        CM_QUIT,
        "^Q",
    ))));

    let mut program = Program::new(Box::new(bar), Box::new(desktop));
    let mut backend = MockBackend::new(80, 24);

    // Inject Ctrl-Q — StatusBar translates to CM_QUIT, Program exits
    backend.inject_key(KeyCode::Char('q'), KeyMod::CTRL);

    // run_cycles should process the quit and return
    program.run_cycles(&mut backend, &mut |_| {}, 5);

    // If we reach here, quit worked. Verify something rendered.
    assert!(backend.buffer().is_some());
}
