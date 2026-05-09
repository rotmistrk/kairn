//! App — top-level view. Creates desktop + status bar, handles commands.

use std::path::PathBuf;

use txv_core::prelude::*;

use crate::broker::{FileBroker, OpenResult};
use crate::commands::*;
use crate::completer::CommandCompleter;
use crate::desktop::{SlotId, SlottedDesktop};
use crate::status::KairnStatusBar;
use crate::views::editor::EditorView;
use crate::views::terminal::TerminalView;
use crate::views::tree::FileTreeView;

/// Root application view.
pub struct App {
    desktop: SlottedDesktop,
    status: KairnStatusBar,
    broker: FileBroker,
    bounds: Rect,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Self {
        let mut desktop = SlottedDesktop::new();

        // Left slot: file tree
        let tree = FileTreeView::new(root_dir);
        desktop.insert_tab(SlotId::Left, "Files", Box::new(tree));

        // Right slot: shell placeholder
        let term = TerminalView::new("Shell");
        desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));

        let mut status = KairnStatusBar::new();
        status.set_completer(Box::new(CommandCompleter));

        Self {
            desktop,
            status,
            broker: FileBroker::new(),
            bounds: Rect::default(),
        }
    }

    fn relayout(&mut self) {
        let b = self.bounds;
        if b.h < 2 {
            return;
        }
        let desktop_rect = Rect::new(b.x, b.y, b.w, b.h - 1);
        self.desktop.set_bounds(desktop_rect);
        let status_rect = Rect::new(b.x, b.y + b.h - 1, b.w, 1);
        self.status.set_bounds(status_rect);
    }

    fn handle_open_file(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let Some(boxed) = data.as_ref() else { return };
        let Some(path) = boxed.downcast_ref::<PathBuf>() else { return };
        let path_str = path.to_string_lossy().to_string();
        let next_tab = self.desktop.tab_count(SlotId::Center);

        match self.broker.open(&path_str, SlotId::Center, next_tab) {
            OpenResult::AlreadyOpen { slot, tab } => {
                self.desktop.focus_tab(slot, tab);
            }
            OpenResult::Opened => {
                if let Ok(editor) = EditorView::open(path) {
                    let title = editor.title().to_string();
                    self.desktop.insert_tab(
                        SlotId::Center,
                        title,
                        Box::new(editor),
                    );
                }
            }
        }
    }

    fn handle_file_deleted(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let Some(boxed) = data.as_ref() else { return };
        let Some(path) = boxed.downcast_ref::<String>() else { return };
        self.broker.close(path);
        // Close the editor tab showing this file
        let filename = std::path::Path::new(path.as_str())
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        self.desktop.close_tab_by_title(SlotId::Center, filename);
    }

    fn handle_execute_command(
        &mut self,
        data: &Option<Box<dyn std::any::Any + Send>>,
        queue: &mut EventQueue,
    ) {
        let Some(boxed) = data.as_ref() else { return };
        let Some(text) = boxed.downcast_ref::<String>() else { return };
        let parts: Vec<&str> = text.trim().splitn(2, ' ').collect();
        let cmd = parts.first().copied().unwrap_or("");
        let arg = parts.get(1).copied().unwrap_or("");

        match cmd {
            "help" => queue.put_command(CM_SHOW_HELP, None),
            "quit" => queue.put_command(CM_QUIT, None),
            "open" => {
                if !arg.is_empty() {
                    let path = PathBuf::from(arg);
                    queue.put_command(CM_OPEN_FILE, Some(Box::new(path)));
                }
            }
            "save" => queue.put_command(CM_SAVE, None),
            "close" => queue.put_command(CM_TAB_CLOSE, None),
            "shell" => queue.put_command(CM_NEW_SHELL, None),
            _ => {}
        }
    }

    fn handle_command(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
        queue: &mut EventQueue,
    ) {
        match id {
            CM_OPEN_FILE => self.handle_open_file(data),
            CM_FILE_DELETED => self.handle_file_deleted(data),
            CM_EXECUTE_COMMAND => self.handle_execute_command(data, queue),
            CM_NEW_SHELL => {
                let term = TerminalView::new("Shell");
                self.desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));
            }
            _ => {}
        }
    }
}

impl View for App {
    fn bounds(&self) -> Rect { self.bounds }

    fn set_bounds(&mut self, r: Rect) {
        self.bounds = r;
        self.relayout();
    }

    fn options(&self) -> ViewOptions {
        ViewOptions { focusable: true, ..ViewOptions::default() }
    }

    fn title(&self) -> &str { "kairn" }
    fn needs_redraw(&self) -> bool {
        self.desktop.needs_redraw() || self.status.needs_redraw()
    }
    fn mark_redrawn(&mut self) {
        self.desktop.mark_redrawn();
        self.status.mark_redrawn();
    }
    fn select(&mut self) { self.desktop.select(); }
    fn unselect(&mut self) { self.desktop.unselect(); }

    fn draw(&self, surface: &mut Surface) {
        self.desktop.draw(surface);
        self.status.draw(surface);
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        // Resize
        if let Event::Resize(w, h) = event {
            self.set_bounds(Rect::new(0, 0, *w, *h));
            return HandleResult::Consumed;
        }

        // Status bar preprocesses keys (F1-F5, Alt-x, Ctrl-Q) and prompt mode
        if self.status.handle(event, queue) == HandleResult::Consumed {
            // Process any commands emitted by status bar
            let events = queue.drain();
            for ev in events {
                if let Event::Command { id, ref data } = ev {
                    self.handle_command(id, data, queue);
                }
            }
            return HandleResult::Consumed;
        }

        // Desktop handles the rest
        if self.desktop.handle(event, queue) == HandleResult::Consumed {
            // Process any commands emitted by views inside desktop
            let events = queue.drain();
            for ev in events {
                if let Event::Command { id, ref data } = ev {
                    self.handle_command(id, data, queue);
                }
            }
            return HandleResult::Consumed;
        }

        // Process commands emitted by children
        let events = queue.drain();
        for ev in events {
            if let Event::Command { id, ref data } = ev {
                self.handle_command(id, data, queue);
            }
            queue.put(ev);
        }

        HandleResult::Ignored
    }
}
