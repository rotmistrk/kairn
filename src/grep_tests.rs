use super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn grep_finds_matches() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
    let state = grep_async("main", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    assert!(!entries.is_empty());
    assert!(state.is_done());
}

#[test]
fn grep_case_insensitive() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.rs"), "Hello World\nhello world\n").unwrap();
    let state = grep_async("-i HELLO", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    assert_eq!(entries.len(), 2);
}

#[test]
fn grep_fixed_string() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "a.b\na+b\n").unwrap();
    // -F treats "a.b" as literal, not regex (. matches any)
    let state = grep_async("-F a.b", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].text.contains("a.b"));
}

#[test]
fn grep_word_boundary() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.rs"), "fn foo()\nfn foobar()\n").unwrap();
    let state = grep_async("-w foo", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].text.contains("fn foo()"));
}

#[test]
fn grep_quoted_pattern() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello world\nhello\nworld\n").unwrap();
    let state = grep_async("\"hello world\"", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    assert_eq!(entries.len(), 1);
}

#[test]
fn grep_combined_flags() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.rs"), "FooBar\nfoo_bar\nfoo\n").unwrap();
    let state = grep_async("-iw foo", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    // matches "foo" (word boundary + case insensitive), not "FooBar" or "foo_bar"
    assert_eq!(entries.len(), 1);
    assert!(entries[0].text.contains("foo"));
}

#[test]
fn grep_respects_gitignore() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::write(dir.path().join(".gitignore"), "ignored/\n").unwrap();
    fs::create_dir(dir.path().join("ignored")).unwrap();
    fs::write(dir.path().join("ignored/file.rs"), "findme\n").unwrap();
    fs::write(dir.path().join("visible.rs"), "findme\n").unwrap();
    let state = grep_async("findme", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].path.ends_with("visible.rs"));
}

#[test]
fn grep_regex_alternation() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "apple\nbanana\ncherry\n").unwrap();
    let state = grep_async("-E \"apple|cherry\"", dir.path(), Waker::noop());
    std::thread::sleep(std::time::Duration::from_millis(100));
    let entries = state.take_entries();
    assert_eq!(entries.len(), 2);
}

#[test]
fn parse_args_error_on_bad_flag() {
    let result = parse_grep_args("-Z pattern");
    assert!(result.is_err());
}

#[test]
fn parse_args_no_pattern() {
    let result = parse_grep_args("-i");
    assert!(result.is_err());
}
