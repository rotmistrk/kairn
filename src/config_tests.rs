//! Tests for config loading.
// Marker for mcp-lint: #[cfg(test)]

use std::fs;
use std::path::{Path, PathBuf};

use super::*;

fn write_config(dir: &Path, content: &str) -> PathBuf {
    let file = dir.join("init.tcl");
    fs::write(&file, content).unwrap();
    file
}

#[test]
fn default_settings_when_no_config_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("nonexistent.tcl");
    let s = load_config_from(&path);
    assert!(s.editor_defaults.wrap);
    assert!(!s.editor_defaults.list);
    assert_eq!(s.editor_defaults.tabstop, 4);
    assert!(s.editor_defaults.number);
    assert_eq!(s.clock_interval, 60);
}

#[test]
fn config_sets_editor_wrap_off() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), "set editor.wrap off");
    let s = load_config_from(&path);
    assert!(!s.editor_defaults.wrap);
}

#[test]
fn config_sets_tabstop() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), "set editor.tabstop 8");
    let s = load_config_from(&path);
    assert_eq!(s.editor_defaults.tabstop, 8);
}

#[test]
fn config_sets_clock_interval() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), "set clock.interval 30");
    let s = load_config_from(&path);
    assert_eq!(s.clock_interval, 30);
}

#[test]
fn config_ignores_unknown_variables() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), "set unknown.thing foo");
    let s = load_config_from(&path);
    assert_eq!(s.clock_interval, 60);
    assert!(s.editor_defaults.wrap);
}

#[test]
fn config_handles_syntax_error_gracefully() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), "{{{");
    let s = load_config_from(&path);
    assert_eq!(s.clock_interval, 60);
    assert!(s.editor_defaults.wrap);
}

#[test]
fn config_multiple_settings() {
    let tmp = tempfile::tempdir().unwrap();
    let script = "set editor.wrap off\n\
                  set editor.list on\n\
                  set editor.tabstop 2\n\
                  set editor.number off\n\
                  set clock.interval 120";
    let path = write_config(tmp.path(), script);
    let s = load_config_from(&path);
    assert!(!s.editor_defaults.wrap);
    assert!(s.editor_defaults.list);
    assert_eq!(s.editor_defaults.tabstop, 2);
    assert!(!s.editor_defaults.number);
    assert_eq!(s.clock_interval, 120);
}

#[test]
fn config_sets_layout_thresholds() {
    let tmp = tempfile::tempdir().unwrap();
    let script = "set layout.wide-threshold 250\nset layout.tall-threshold 180";
    let path = write_config(tmp.path(), script);
    let s = load_config_from(&path);
    assert_eq!(s.layout_wide_threshold, 250);
    assert_eq!(s.layout_tall_threshold, 180);
}

#[test]
fn config_sets_kiro_cmd_list() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), r#"set kiro.cmd %[kiro-cli, chat, --tui]"#);
    let s = load_config_from(&path);
    assert_eq!(s.kiro.cmd, vec!["kiro-cli", "chat", "--tui"]);
}

#[test]
fn config_sets_kiro_resume_first() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), r#"set kiro.resume-first %[--resume]"#);
    let s = load_config_from(&path);
    assert_eq!(s.kiro.resume_first, vec!["--resume"]);
}

#[test]
fn config_sets_kiro_resume_rest() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), r#"set kiro.resume-rest %[--resume-picker]"#);
    let s = load_config_from(&path);
    assert_eq!(s.kiro.resume_rest, vec!["--resume-picker"]);
}

#[test]
fn config_kiro_cmd_default_without_setting() {
    let tmp = tempfile::tempdir().unwrap();
    let path = write_config(tmp.path(), "set editor.wrap off");
    let s = load_config_from(&path);
    assert_eq!(s.kiro.cmd, vec!["kiro-cli", "chat"]);
    assert_eq!(s.kiro.resume_first, vec!["--resume"]);
    assert_eq!(s.kiro.resume_rest, vec!["--resume-picker"]);
}
