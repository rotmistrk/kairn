//! Tests for todo file timestamp tracking and reload-on-change.

use std::fs;
use std::thread;
use std::time::Duration;

use tempfile::TempDir;

use kairn::views::todo_tree::data::TodoTreeData;

fn setup_todo(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join(".kairn.todo");
    let content =
        r#"{"version":"2.0","title":"Test","items":[{"id":"a","title":"Task A","completed":"Open","items":[]}]}"#;
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn reload_detects_external_change() {
    let dir = TempDir::new().unwrap();
    let path = setup_todo(&dir);
    let mut data = TodoTreeData::new(&path);
    assert_eq!(data.file().items.len(), 1);
    assert_eq!(data.file().items[0].title, "Task A");

    // Simulate external modification
    thread::sleep(Duration::from_millis(50));
    let new_content = r#"{"version":"2.0","title":"Test","items":[{"id":"a","title":"Task A","completed":"Open","items":[]},{"id":"b","title":"Task B","completed":"Open","items":[]}]}"#;
    fs::write(&path, new_content).unwrap();

    assert!(data.reload_if_changed());
    assert_eq!(data.file().items.len(), 2);
    assert_eq!(data.file().items[1].title, "Task B");
}

#[test]
fn reload_returns_false_when_unchanged() {
    let dir = TempDir::new().unwrap();
    let path = setup_todo(&dir);
    let mut data = TodoTreeData::new(&path);

    assert!(!data.reload_if_changed());
}

#[test]
fn save_updates_mtime() {
    let dir = TempDir::new().unwrap();
    let path = setup_todo(&dir);
    let mut data = TodoTreeData::new(&path);

    // Modify in memory and save
    data.file_mut().items[0].title = "Modified".to_string();
    data.save();

    // After save, no external change detected
    assert!(!data.reload_if_changed());

    // File on disk has the new content
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("Modified"));
}

#[test]
fn save_reloads_before_writing_if_file_changed() {
    let dir = TempDir::new().unwrap();
    let path = setup_todo(&dir);
    let mut data = TodoTreeData::new(&path);

    // Simulate external modification
    thread::sleep(Duration::from_millis(50));
    let new_content =
        r#"{"version":"2.0","title":"Test","items":[{"id":"x","title":"External","completed":"Open","items":[]}]}"#;
    fs::write(&path, new_content).unwrap();

    // Now save — it should reload first, then save our (reloaded) state
    data.save();

    // The file should contain the reloaded content (since reload overwrites in-memory)
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("External"));
}
