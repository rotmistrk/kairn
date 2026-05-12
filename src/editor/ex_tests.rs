use super::*;

#[test]
fn test_parse_substitute() {
    let result = parse_ex_full("%s/foo/bar/g", 0, 10);
    assert_eq!(
        result,
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
fn test_parse_delete_range() {
    let result = parse_ex_full("1,3d", 0, 10);
    assert_eq!(result, Some(ExCommand::Delete { start: 0, end: 2 }));
}

#[test]
fn test_parse_yank() {
    let result = parse_ex_full("%y", 0, 5);
    assert_eq!(result, Some(ExCommand::Yank { start: 0, end: 4 }));
}

#[test]
fn test_diff_not_delete() {
    assert_eq!(parse_ex_full("diff", 0, 10), Some(ExCommand::Diff(String::new())));
    assert_eq!(parse_ex_full("dif", 0, 10), Some(ExCommand::Diff(String::new())));
    assert_eq!(
        parse_ex_full("diff HEAD~1", 0, 10),
        Some(ExCommand::Diff("HEAD~1".to_string()))
    );
}

#[test]
fn test_d_is_delete() {
    assert_eq!(parse_ex_full("d", 0, 10), Some(ExCommand::Delete { start: 0, end: 0 }));
    assert_eq!(parse_ex_full("de", 0, 10), Some(ExCommand::Delete { start: 0, end: 0 }));
    assert_eq!(
        parse_ex_full("del", 0, 10),
        Some(ExCommand::Delete { start: 0, end: 0 })
    );
}

#[test]
fn test_nodiff() {
    assert_eq!(parse_ex_full("nodiff", 0, 10), Some(ExCommand::NoDiff));
    assert_eq!(parse_ex_full("nod", 0, 10), Some(ExCommand::NoDiff));
}

#[test]
fn test_edit() {
    assert_eq!(
        parse_ex_full("e foo.rs", 0, 10),
        Some(ExCommand::Edit("foo.rs".to_string()))
    );
    assert_eq!(
        parse_ex_full("edit foo.rs", 0, 10),
        Some(ExCommand::Edit("foo.rs".to_string()))
    );
}

#[test]
fn test_quit_force() {
    assert_eq!(parse_ex_full("q!", 0, 10), Some(ExCommand::QuitForce));
}

#[test]
fn test_set_and_setglobal() {
    assert_eq!(parse_ex_full("set nu", 0, 10), Some(ExCommand::Set("nu".to_string())));
    assert_eq!(parse_ex_full("se nu", 0, 10), Some(ExCommand::Set("nu".to_string())));
    assert_eq!(
        parse_ex_full("setg nu", 0, 10),
        Some(ExCommand::SetGlobal("nu".to_string()))
    );
    assert_eq!(
        parse_ex_full("setglobal nu", 0, 10),
        Some(ExCommand::SetGlobal("nu".to_string()))
    );
}

#[test]
fn test_parse_relative_range() {
    let result = parse_ex_full(".,+2d", 1, 5);
    assert_eq!(result, Some(ExCommand::Delete { start: 1, end: 3 }));
    let result = parse_ex_full(".,+2y", 1, 5);
    assert_eq!(result, Some(ExCommand::Yank { start: 1, end: 3 }));
}
