//! Test: Hook registry fires hooks in order with correct filtering.

use kairn::scripting::hooks::{HookEvent, HookRegistry};

#[test]
fn hook_fires_on_matching_filter() {
    let mut reg = HookRegistry::new();
    reg.add(HookEvent::CharInserted, Some("("), "puts matched".into())
        .unwrap();
    let scripts = reg.fire(&HookEvent::CharInserted, "(");
    assert_eq!(scripts.len(), 1);
    assert_eq!(scripts[0], "puts matched");
}

#[test]
fn hook_does_not_fire_on_non_matching_filter() {
    let mut reg = HookRegistry::new();
    reg.add(HookEvent::CharInserted, Some("("), "puts matched".into())
        .unwrap();
    let scripts = reg.fire(&HookEvent::CharInserted, "x");
    assert!(scripts.is_empty());
}

#[test]
fn star_filter_matches_everything() {
    let mut reg = HookRegistry::new();
    reg.add(HookEvent::CharInserted, Some("*"), "puts any".into()).unwrap();
    let scripts = reg.fire(&HookEvent::CharInserted, "z");
    assert_eq!(scripts.len(), 1);
}

#[test]
fn multiple_hooks_fire_in_order() {
    let mut reg = HookRegistry::new();
    reg.add(HookEvent::CharInserted, Some("*"), "first".into()).unwrap();
    reg.add(HookEvent::CharInserted, Some("*"), "second".into()).unwrap();
    reg.add(HookEvent::CharInserted, Some("*"), "third".into()).unwrap();
    let scripts = reg.fire(&HookEvent::CharInserted, "a");
    assert_eq!(scripts, vec!["first", "second", "third"]);
}

#[test]
fn hook_without_filter_always_fires() {
    let mut reg = HookRegistry::new();
    reg.add(HookEvent::FileSave, None, "build test".into()).unwrap();
    let scripts = reg.fire(&HookEvent::FileSave, "");
    assert_eq!(scripts.len(), 1);
}

#[test]
fn glob_filter_matches_pattern() {
    let mut reg = HookRegistry::new();
    reg.add(HookEvent::FileSave, Some("*.rs"), "format".into()).unwrap();
    assert_eq!(reg.fire(&HookEvent::FileSave, "main.rs").len(), 1);
    assert_eq!(reg.fire(&HookEvent::FileSave, "main.go").len(), 0);
}

#[test]
fn glob_filter_with_special_chars() {
    let mut reg = HookRegistry::new();
    // Paths with regex-special chars like ++ should work fine as glob
    reg.add(HookEvent::FileOpen, Some("/home/user/kairn++/*.rs"), "hook".into())
        .unwrap();
    assert_eq!(reg.fire(&HookEvent::FileOpen, "/home/user/kairn++/main.rs").len(), 1);
    assert_eq!(reg.fire(&HookEvent::FileOpen, "/home/user/other/main.rs").len(), 0);
}

#[test]
fn remove_hooks_for_event() {
    let mut reg = HookRegistry::new();
    reg.add(HookEvent::CharInserted, Some("*"), "hook1".into()).unwrap();
    reg.add(HookEvent::FileSave, None, "hook2".into()).unwrap();
    reg.remove(&HookEvent::CharInserted);
    assert!(reg.fire(&HookEvent::CharInserted, "x").is_empty());
    assert_eq!(reg.fire(&HookEvent::FileSave, "").len(), 1);
}
