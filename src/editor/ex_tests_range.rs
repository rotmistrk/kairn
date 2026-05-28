use super::*;

// --- Range: % ---

#[test]
fn range_percent() {
    assert_eq!(parse_ex_full("%d", 0, 10), Some(ExCommand::Delete { start: 0, end: 9 }));
    assert_eq!(parse_ex_full("%y", 0, 5), Some(ExCommand::Yank { start: 0, end: 4 }));
}

// --- Range: dot ---

#[test]
fn range_dot() {
    assert_eq!(parse_ex_full(".d", 4, 10), Some(ExCommand::Delete { start: 4, end: 4 }));
}

// --- Range: $ ---

#[test]
fn range_dollar() {
    assert_eq!(parse_ex_full("$d", 0, 10), Some(ExCommand::Delete { start: 9, end: 9 }));
}

// --- Range: line number ---

#[test]
fn range_line_number() {
    assert_eq!(parse_ex_full("3d", 0, 10), Some(ExCommand::Delete { start: 2, end: 2 }));
}

// --- Range: N,M ---

#[test]
fn range_n_comma_m() {
    assert_eq!(parse_ex_full("1,3d", 0, 10), Some(ExCommand::Delete { start: 0, end: 2 }));
    assert_eq!(parse_ex_full("2,4d", 0, 10), Some(ExCommand::Delete { start: 1, end: 3 }));
}

// --- Range: .,+N and .,-N ---

#[test]
fn range_dot_plus_n() {
    assert_eq!(parse_ex_full(".,+2d", 1, 5), Some(ExCommand::Delete { start: 1, end: 3 }));
    assert_eq!(parse_ex_full(".,+2y", 1, 5), Some(ExCommand::Yank { start: 1, end: 3 }));
}

#[test]
fn range_dot_minus_n() {
    assert_eq!(parse_ex_full(".-2,.d", 4, 10), Some(ExCommand::Delete { start: 2, end: 4 }));
}

// --- Range: +N / -N (relative to cursor) ---

#[test]
fn range_plus_n() {
    assert_eq!(parse_ex_full("+3d", 2, 10), Some(ExCommand::Delete { start: 5, end: 5 }));
}

#[test]
fn range_minus_n() {
    assert_eq!(parse_ex_full("-2d", 5, 10), Some(ExCommand::Delete { start: 3, end: 3 }));
}

// --- Range: empty = current line ---

#[test]
fn range_empty_is_current_line() {
    assert_eq!(parse_ex_full("d", 7, 10), Some(ExCommand::Delete { start: 7, end: 7 }));
}

// --- Range: comma with empty parts ---

#[test]
fn range_comma_empty_parts() {
    assert_eq!(
        parse_ex_full(",s/ pub//", 3, 10),
        Some(ExCommand::Substitute {
            start: 3,
            end: 3,
            pattern: " pub".to_string(),
            replacement: String::new(),
            global: false,
        })
    );
}

// --- Range clamping ---

#[test]
fn range_plus_clamps_to_last_line() {
    assert_eq!(parse_ex_full("+99d", 0, 5), Some(ExCommand::Delete { start: 4, end: 4 }));
}

#[test]
fn range_minus_clamps_to_zero() {
    assert_eq!(parse_ex_full("-99d", 2, 10), Some(ExCommand::Delete { start: 0, end: 0 }));
}

// --- Substitute variations ---

#[test]
fn substitute_basic() {
    assert_eq!(
        parse_ex_full("%s/foo/bar/g", 0, 10),
        Some(ExCommand::Substitute {
            start: 0,
            end: 9,
            pattern: "foo".to_string(),
            replacement: "bar".to_string(),
            global: true,
        })
    );
}

#[test]
fn substitute_no_global_flag() {
    assert_eq!(
        parse_ex_full("%s/foo/bar/", 0, 10),
        Some(ExCommand::Substitute {
            start: 0,
            end: 9,
            pattern: "foo".to_string(),
            replacement: "bar".to_string(),
            global: false,
        })
    );
}

#[test]
fn substitute_empty_replacement() {
    assert_eq!(
        parse_ex_full("%s/pub//", 0, 10),
        Some(ExCommand::Substitute {
            start: 0,
            end: 9,
            pattern: "pub".to_string(),
            replacement: String::new(),
            global: false,
        })
    );
}

#[test]
fn substitute_space_in_pattern() {
    assert_eq!(
        parse_ex_full("%s/ pub//", 0, 10),
        Some(ExCommand::Substitute {
            start: 0,
            end: 9,
            pattern: " pub".to_string(),
            replacement: String::new(),
            global: false,
        })
    );
}

#[test]
fn substitute_different_delimiter() {
    assert_eq!(
        parse_ex_full("%s#foo#bar#g", 0, 10),
        Some(ExCommand::Substitute {
            start: 0,
            end: 9,
            pattern: "foo".to_string(),
            replacement: "bar".to_string(),
            global: true,
        })
    );
}

#[test]
fn substitute_current_line_no_range() {
    assert_eq!(
        parse_ex_full("s/a/b/", 3, 10),
        Some(ExCommand::Substitute {
            start: 3,
            end: 3,
            pattern: "a".to_string(),
            replacement: "b".to_string(),
            global: false,
        })
    );
}

// --- Shell filter ---

#[test]
fn shell_filter_percent_range() {
    assert_eq!(
        parse_ex_full("%!sort", 0, 10),
        Some(ExCommand::Shell {
            start: 0,
            end: 9,
            command: "sort".to_string(),
        })
    );
}

#[test]
fn shell_filter_line_range() {
    assert_eq!(
        parse_ex_full("1,5!fmt", 0, 10),
        Some(ExCommand::Shell {
            start: 0,
            end: 4,
            command: "fmt".to_string(),
        })
    );
}

#[test]
fn substitute_comma_range_empty_parts() {
    // ",s/ pub//" — comma alone means current line to current line
    assert_eq!(
        parse_ex_full(",s/ pub//", 3, 10),
        Some(ExCommand::Substitute {
            start: 3,
            end: 3,
            pattern: " pub".to_string(),
            replacement: String::new(),
            global: false,
        })
    );
}
