//! Tests for diff mode data model — virtual lines, folding, navigation.

use kairn::views::editor::diff_model::*;

#[test]
fn no_changes() {
    let opts = DiffOpts { base: "HEAD".into(), context: usize::MAX, ignore_ws: false };
    let lines = build_diff_lines("a\nb\n", "a\nb\n", &opts);
    assert_eq!(lines.len(), 2);
    assert!(matches!(lines[0], DiffLine::Context { buf_line: 0, base_line: 0 }));
}

#[test]
fn added_line() {
    let opts = DiffOpts { base: "HEAD".into(), context: usize::MAX, ignore_ws: false };
    let lines = build_diff_lines("a\n", "a\nb\n", &opts);
    assert_eq!(lines.len(), 2);
    assert!(matches!(lines[1], DiffLine::Added { buf_line: 1 }));
}

#[test]
fn deleted_line() {
    let opts = DiffOpts { base: "HEAD".into(), context: usize::MAX, ignore_ws: false };
    let lines = build_diff_lines("a\nb\n", "a\n", &opts);
    assert_eq!(lines.len(), 2);
    if let DiffLine::Deleted { text, base_line } = &lines[1] {
        assert_eq!(text, "b");
        assert_eq!(*base_line, 1);
    } else {
        panic!("expected Deleted");
    }
}

#[test]
fn fold_hides_distant_context() {
    let opts = DiffOpts { base: "HEAD".into(), context: 1, ignore_ws: false };
    let base = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n";
    let current = "1\n2\n3\n4\n5\nNEW\n6\n7\n8\n9\n10\n";
    let lines = build_diff_lines(base, current, &opts);
    assert!(matches!(lines[0], DiffLine::Folded { count: 4 }));
    assert!(matches!(lines[1], DiffLine::Context { .. }));
    assert!(matches!(lines[2], DiffLine::Added { .. }));
    assert!(matches!(lines[3], DiffLine::Context { .. }));
    assert!(matches!(lines[4], DiffLine::Folded { count: 4 }));
}

#[test]
fn cursor_buf_line_on_deleted() {
    let lines = vec![
        DiffLine::Context { buf_line: 0, base_line: 0 },
        DiffLine::Deleted { text: "x".into(), base_line: 1 },
        DiffLine::Context { buf_line: 1, base_line: 2 },
    ];
    let ds = DiffState {
        lines, scroll: 0, cursor: 1,
        base_ref: "HEAD".into(), context_lines: usize::MAX, ignore_ws: false,
    };
    assert_eq!(ds.cursor_buf_line(), 0);
}

#[test]
fn next_prev_hunk() {
    let lines = vec![
        DiffLine::Context { buf_line: 0, base_line: 0 },
        DiffLine::Added { buf_line: 1 },
        DiffLine::Context { buf_line: 2, base_line: 1 },
        DiffLine::Deleted { text: "x".into(), base_line: 2 },
        DiffLine::Context { buf_line: 3, base_line: 3 },
    ];
    let mut ds = DiffState {
        lines, scroll: 0, cursor: 0,
        base_ref: "HEAD".into(), context_lines: usize::MAX, ignore_ws: false,
    };
    assert_eq!(ds.next_hunk(), Some(1));
    ds.cursor = 1;
    assert_eq!(ds.next_hunk(), Some(3));
    ds.cursor = 3;
    assert_eq!(ds.next_hunk(), None);
    assert_eq!(ds.prev_hunk(), Some(1));
}

#[test]
fn parse_args() {
    let opts = parse_diff_args("-U5 -w main");
    assert_eq!(opts.base, "main");
    assert_eq!(opts.context, 5);
    assert!(opts.ignore_ws);
}

#[test]
fn ignore_whitespace() {
    let opts = DiffOpts { base: "HEAD".into(), context: usize::MAX, ignore_ws: true };
    let lines = build_diff_lines("a  b\n", "a b\n", &opts);
    // Should be context (whitespace ignored)
    assert_eq!(lines.len(), 1);
    assert!(matches!(lines[0], DiffLine::Context { .. }));
}
