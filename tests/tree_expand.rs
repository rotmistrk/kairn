//! Test that right arrow on expanded directory does not emit CM_OPEN_FILE.

use kairn::commands::CM_OPEN_FILE;
use kairn::views::tree::FileTreeView;
use txv_core::prelude::*;

#[test]
fn right_arrow_on_expanded_dir_does_not_open_file() {
    let tmp = tempfile::tempdir().unwrap();
    let sub = tmp.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("file.txt"), "hello").unwrap();

    let sink = EventSink::new();
    let mut view = FileTreeView::new(tmp.path().to_path_buf(), None);
    view.set_bounds(Rect::new(0, 0, 40, 10));
    view.set_sink(sink.clone());

    let right = Event::Key(KeyEvent {
        code: KeyCode::Right,
        modifiers: KeyMod::default(),
    });
    view.handle(&right);
    let events = sink.drain();
    assert!(!events
        .iter()
        .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_OPEN_FILE)));

    view.handle(&right);
    let events = sink.drain();
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_OPEN_FILE)),
        "CM_OPEN_FILE should not be emitted for directories"
    );
}
