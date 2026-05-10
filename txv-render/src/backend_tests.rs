use super::*;
use txv_core::cell::Style;
use txv_core::surface::Surface;

#[test]
fn diff_detects_all_changed_cells() {
    let mut s1 = Surface::new(10, 5);
    s1.print(0, 0, "hello", Style::default());
    s1.print(0, 1, "world", Style::default());

    // Simulate: previous = copy of s1
    let mut prev = Surface::new(10, 5);
    for y in 0..5 {
        for x in 0..10 {
            let c = s1.cell(x, y);
            prev.put(x, y, c.ch, c.style);
        }
    }

    // Draw different content
    let mut s2 = Surface::new(10, 5);
    s2.fill(' ', Style::default());
    s2.print(0, 0, "HELLO", Style::default());
    s2.print(0, 1, "WORLD", Style::default());

    let changed = diff_cells(&s2, &prev);

    // All 10 chars on rows 0-1 changed (lowercase → uppercase)
    for x in 0..5u16 {
        assert!(changed.contains(&(x, 0)), "({},0) should be changed", x);
        assert!(changed.contains(&(x, 1)), "({},1) should be changed", x);
    }
}

#[test]
fn diff_detects_style_changes() {
    let mut s1 = Surface::new(10, 1);
    s1.print(0, 0, "test", Style::default());

    let mut prev = Surface::new(10, 1);
    for x in 0..10 {
        let c = s1.cell(x, 0);
        prev.put(x, 0, c.ch, c.style);
    }

    // Same chars but different style
    let mut s2 = Surface::new(10, 1);
    let bold = Style {
        attrs: Attrs {
            bold: true,
            ..Attrs::default()
        },
        ..Style::default()
    };
    s2.print(0, 0, "test", bold);

    let changed = diff_cells(&s2, &prev);
    assert_eq!(changed.len(), 4, "4 cells changed style");
}

#[test]
fn previous_buffer_updated_after_flush_simulation() {
    // Simulate the flush copy logic
    let mut prev = Surface::new(10, 3);

    // Frame 1: draw "AAAA"
    let mut frame1 = Surface::new(10, 3);
    frame1.fill(' ', Style::default());
    frame1.print(0, 0, "AAAA", Style::default());

    // Copy frame1 → prev (simulating end of flush)
    for y in 0..3 {
        for x in 0..10 {
            let c = frame1.cell(x, y);
            prev.put(x, y, c.ch, c.style);
        }
    }

    // Frame 2: draw "BB" (shorter)
    let mut frame2 = Surface::new(10, 3);
    frame2.fill(' ', Style::default());
    frame2.print(0, 0, "BB", Style::default());

    let changed = diff_cells(&frame2, &prev);

    // Cells 0,1 changed (A→B), cells 2,3 changed (A→space)
    assert!(changed.contains(&(0, 0)), "cell 0 should change A→B");
    assert!(changed.contains(&(1, 0)), "cell 1 should change A→B");
    assert!(changed.contains(&(2, 0)), "cell 2 should change A→space");
    assert!(changed.contains(&(3, 0)), "cell 3 should change A→space");
}
