use super::truncate_title;

#[test]
fn short_title_unchanged() {
    assert_eq!(truncate_title("main.rs", 20), "main.rs");
}

#[test]
fn path_collapses_leading_segments() {
    let t = "/home/user/.cargo/registry/src/crates.io/crossterm-0.28.1/src/style.rs";
    let result = truncate_title(t, 40);
    assert!(result.starts_with("…/"));
    assert!(result.ends_with("style.rs"));
    assert!(result.chars().count() <= 40);
}

#[test]
fn relative_path_collapses() {
    assert_eq!(
        truncate_title("src/views/editor/handle_completion.rs", 30),
        "…/editor/handle_completion.rs"
    );
}

#[test]
fn non_path_truncates_with_ellipsis() {
    assert_eq!(truncate_title("very long title without slashes", 10), "very long…");
}
