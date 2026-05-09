//! App — top-level Group. Uses GroupState for three-phase dispatch.
//! StatusBar (preprocess) + Desktop (focused) are children.

use std::path::PathBuf;

use txv_core::prelude::*;

use crate::broker::{FileBroker, OpenResult};
use crate::commands::*;
use crate::completer::CommandCompleter;
use crate::desktop::{SlotId, SlottedDesktop};
use crate::status::KairnStatusBar;
use crate::views::editor::EditorView;
use crate::views::help::HelpView;
use crate::views::terminal::TerminalView;
use crate::views::tree::FileTreeView;

/// Root application — a Group with StatusBar + Desktop.
pub struct App {
    group: GroupState,
    broker: FileBroker,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Self {
        let mut desktop = SlottedDesktop::new();
        let tree = FileTreeView::new(root_dir);
        desktop.insert_tab(SlotId::Left, "Files", Box::new(tree));
        let term = TerminalView::new("Shell");
        desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));

        let mut status = KairnStatusBar::new();
        status.set_completer(Box::new(CommandCompleter));

        let mut group = GroupState::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        });
        group.insert(Box::new(status));  // child 0: preprocess
        group.insert(Box::new(desktop)); // child 1: focused
        group.focused = 1;

        Self {
            group,
            broker: FileBroker::new(),
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
            CM_SHOW_HELP => {
                if let Some(desktop) = self.desktop_mut() {
                    let help = HelpView::new();
                    desktop.insert_tab(SlotId::Center, "Help", Box::new(help));
                }
            }
            CM_NEW_SHELL => {
                let term = TerminalView::new("Shell");
                if let Some(desktop) = self.desktop_mut() {
                    desktop.insert_tab(SlotId::Right, "Shell", Box::new(term));
                }
            }
            _ => {}
        }
    }

    fn handle_open_file(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let Some(boxed) = data.as_ref() else { return };
        let Some(path) = boxed.downcast_ref::<PathBuf>() else { return };
        let path_str = path.to_string_lossy().to_string();

        // Check broker first (no borrow on group)
        let tab_count = 0; // approximate — broker just needs a slot
        let open_result = self.broker.open(&path_str, SlotId::Center, tab_count);

        match open_result {
            OpenResult::AlreadyOpen { slot, tab } => {
                if let Some(desktop) = self.desktop_mut() {
                    desktop.focus_tab(slot, tab);
                }
            }
            OpenResult::Opened => {
                if let Ok(editor) = EditorView::open(path) {
                    let title = editor.title().to_string();
                    if let Some(desktop) = self.desktop_mut() {
                        desktop.insert_tab(SlotId::Center, title, Box::new(editor));
                    }
                }
            }
        }
    }

    fn handle_file_deleted(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let Some(boxed) = data.as_ref() else { return };
        let Some(path) = boxed.downcast_ref::<String>() else { return };
        self.broker.close(path);
        let filename = std::path::Path::new(path.as_str())
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        if let Some(desktop) = self.desktop_mut() {
            desktop.close_tab_by_title(SlotId::Center, filename);
        }
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
            "open" if !arg.is_empty() => {
                queue.put_command(CM_OPEN_FILE, Some(Box::new(PathBuf::from(arg))));
            }
            "save" => queue.put_command(CM_SAVE, None),
            "close" => queue.put_command(CM_TAB_CLOSE, None),
            "shell" => queue.put_command(CM_NEW_SHELL, None),
            _ => {}
        }
    }

    /// Get desktop as SlottedDesktop (downcast from child 1).
    fn desktop_mut(&mut self) -> Option<&mut SlottedDesktop> {
        self.group.children.get_mut(1).and_then(|child| {
            let ptr = child.as_mut() as *mut dyn View;
            // SAFETY: we know child 1 is SlottedDesktop (we inserted it).
            unsafe { (ptr as *mut SlottedDesktop).as_mut() }
        })
    }
}

impl View for App {
    delegate_group_state!(group, override { handle, set_bounds, draw });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.bounds = r;
        self.group.view.dirty = true;
        if r.h >= 2 {
            // Status bar: last row. Desktop: everything else.
            let desktop_rect = Rect::new(r.x, r.y, r.w, r.h - 1);
            let status_rect = Rect::new(r.x, r.y + r.h - 1, r.w, 1);
            if let Some(child) = self.group.children.get_mut(1) {
                child.set_bounds(desktop_rect);
            }
            if let Some(child) = self.group.children.get_mut(0) {
                child.set_bounds(status_rect);
            }
        }
    }

    fn draw(&self, surface: &mut Surface) {
        // Draw desktop first (fills most of screen), then status on top
        for child in &self.group.children {
            child.draw(surface);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Resize(w, h) = event {
            self.set_bounds(Rect::new(0, 0, *w, *h));
            return HandleResult::Consumed;
        }

        // Intercept app-level commands before group dispatch
        if let Event::Command { id, ref data } = event {
            match *id {
                CM_OPEN_FILE | CM_FILE_DELETED | CM_EXECUTE_COMMAND
                | CM_SHOW_HELP | CM_NEW_SHELL => {
                    self.handle_command(*id, data, queue);
                    return HandleResult::Consumed;
                }
                _ => {}
            }
        }

        self.group.dispatch(event, queue)
    }
}
