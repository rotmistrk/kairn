//! FileTreeView — wraps TreeView<FileTreeData>, emits CM_OPEN_FILE on Enter.

use std::collections::HashMap;
use std::path::PathBuf;

use txv_core::cell::Color;
use txv_core::prelude::*;
use txv_widgets::{FileTreeData, TreeView};

use crate::commands::{CM_OPEN_FILE, CM_OPEN_FILE_FOCUS};
use crate::git_status::{collect_git_status, FileStatus};

pub struct FileTreeView {
    inner: TreeView<FileTreeData>,
    last_key_was_right: bool,
    refresh_counter: u16,
    root: PathBuf,
}

impl FileTreeView {
    pub fn new(root: PathBuf) -> Self {
        let data = FileTreeData::new(root.clone());
        let mut view = Self {
            inner: TreeView::new(data),
            last_key_was_right: false,
            refresh_counter: 0,
            root,
        };
        view.update_colors();
        view
    }

    fn update_colors(&mut self) {
        let statuses = collect_git_status(&self.root);
        let colors: HashMap<String, Color> = statuses
            .into_iter()
            .map(|(path, status)| (path, status_color(status)))
            .collect();
        self.inner.data.set_colors(colors);
    }
}

fn status_color(status: FileStatus) -> Color {
    match status {
        FileStatus::Modified => Color::Ansi(12),
        FileStatus::Added => Color::Ansi(2),
        FileStatus::Untracked => Color::Ansi(1),
        FileStatus::Ignored => Color::Ansi(8),
        FileStatus::Clean => Color::Ansi(7),
    }
}

impl View for FileTreeView {
    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }
    fn set_bounds(&mut self, r: Rect) {
        self.inner.set_bounds(r);
    }
    fn options(&self) -> ViewOptions {
        self.inner.options()
    }
    fn title(&self) -> &str {
        "Files"
    }
    fn needs_redraw(&self) -> bool {
        self.inner.needs_redraw()
    }
    fn mark_redrawn(&mut self) {
        self.inner.mark_redrawn();
    }
    fn select(&mut self) {
        self.inner.select();
    }
    fn unselect(&mut self) {
        self.inner.unselect();
    }

    fn draw(&self, surface: &mut Surface) {
        self.inner.draw(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Tick = event {
            self.refresh_counter += 1;
            if self.refresh_counter >= 60 {
                self.refresh_counter = 0;
                self.inner.data.refresh();
            }
            return HandleResult::Ignored;
        }
        if let Event::Key(key) = event {
            self.last_key_was_right = key.code == KeyCode::Right;
        }
        let result = self.inner.handle(event, queue);
        let events = queue.drain();
        for ev in events {
            if let Event::Command { id, data } = &ev {
                if *id == CM_OK {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(&node_id) = boxed.downcast_ref::<usize>() {
                            let path = self.inner.data.path(node_id).to_path_buf();
                            if !path.is_dir() {
                                let cmd = if self.last_key_was_right {
                                    CM_OPEN_FILE_FOCUS
                                } else {
                                    CM_OPEN_FILE
                                };
                                queue.put_command(cmd, Some(Box::new(path)));
                            }
                            continue;
                        }
                    }
                }
            }
            queue.put(ev);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::FileTreeView;
    use crate::commands::CM_OPEN_FILE;
    use txv_core::prelude::*;

    #[test]
    fn right_arrow_on_expanded_dir_does_not_open_file() {
        // Create a temp dir with a subdirectory
        let tmp = tempfile::tempdir().unwrap();
        let sub = tmp.path().join("subdir");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("file.txt"), "hello").unwrap();

        let mut view = FileTreeView::new(tmp.path().to_path_buf());
        view.set_bounds(Rect::new(0, 0, 40, 10));

        let mut queue = EventQueue::new();

        // First Right arrow expands the directory
        let right = Event::Key(KeyEvent {
            code: KeyCode::Right,
            modifiers: KeyMod::default(),
        });
        view.handle(&right, &mut queue);
        // Should not emit CM_OPEN_FILE (just expanded)
        let events: Vec<_> = queue.drain();
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_OPEN_FILE)));

        // Second Right arrow on already-expanded dir should NOT emit CM_OPEN_FILE
        let mut queue = EventQueue::new();
        view.handle(&right, &mut queue);
        let events: Vec<_> = queue.drain();
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_OPEN_FILE)),
            "CM_OPEN_FILE should not be emitted for directories"
        );
    }
}
