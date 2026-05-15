//! Tests for csv_parse module.

use kairn::csv_parse::*;

#[test]
fn detect_comma_delimiter() {
    assert_eq!(detect_delimiter("a,b,c\n1,2,3\n"), ',');
}

#[test]
fn detect_tab_delimiter() {
    assert_eq!(detect_delimiter("a\tb\tc\n1\t2\t3\n"), '\t');
}

#[test]
fn parse_simple_csv() {
    let rows = parse("a,b,c\n1,2,3\n", ',');
    assert_eq!(rows, vec![vec!["a", "b", "c"], vec!["1", "2", "3"]]);
}

#[test]
fn parse_quoted_fields() {
    let rows = parse("\"hello, world\",b\n", ',');
    assert_eq!(rows, vec![vec!["hello, world", "b"]]);
}

#[test]
fn header_detection() {
    let rows = vec![vec!["Name".into(), "Age".into()], vec!["Alice".into(), "30".into()]];
    assert!(detect_header(&rows));
}

#[test]
fn numeric_column_detection() {
    let rows = vec![
        vec!["x".into(), "y".into()],
        vec!["1.5".into(), "hello".into()],
        vec!["2.3".into(), "world".into()],
    ];
    let types = detect_col_types(&rows, true);
    assert!(matches!(types[0], ColType::Numeric { .. }));
    assert!(matches!(types[1], ColType::Text));
}
