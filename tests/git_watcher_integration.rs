//! Git status integration tests.

use kairn::git_status::{dir_status, FileStatus};

#[test]
fn conflict_has_highest_priority() {
    assert_eq!(
        dir_status(&[FileStatus::Modified, FileStatus::Conflict]),
        FileStatus::Conflict
    );
    assert_eq!(
        dir_status(&[FileStatus::Untracked, FileStatus::Conflict]),
        FileStatus::Conflict
    );
}
