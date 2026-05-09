//! App — top-level application with event loop.
//!
//! Owns a SlottedDesktop and a StatusBarView. Runs the event loop,
//! dispatching events to status bar first (preprocess), then desktop.
//! Handles CM_QUIT, CM_OPEN_FILE, CM_SHOW_HELP at the app level.

use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{self, Event as CtEvent};
use crossterm::terminal;
use txv::layout::Rect;
use txv::screen::Screen;
use txv_widgets::view::{DrawContext, Event, HandleResult, View};

use crate::commands::*;
use crate::desktop::SlottedDesktop;
use crate::editor::KeymapKind;
use crate::status::StatusBarView;
use crate::types::{CommandOutbox, OpenFilePayload, PendingCommand, SlotId};
use crate::views::editor::EditorView;
use crate::views::terminal::TerminalView;
use crate::views::tree::FileTreeView;

/// The kairn application.
pub struct App {
    desktop: SlottedDesktop,
    status_bar: StatusBarView,
    screen: Screen,
    running: bool,
    keymap: KeymapKind,
    /// Shared outbox for the tree view to emit commands into.
    tree_outbox: Arc<Mutex<CommandOutbox>>,
}

impl App {
    /// Create a new app with a file tree in the left slot.
    pub fn new(root: &Path) -> Self {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        let mut desktop = SlottedDesktop::new();
        let tree_outbox = Arc::new(Mutex::new(CommandOutbox::default()));

        // Insert file tree in left slot
        if let Some(tree) = FileTreeView::open(root, Arc::clone(&tree_outbox)) {
            desktop.insert_view(SlotId::Left, "Files", Box::new(tree));
        }

        // Insert a shell in the right slot
        if let Some(term) = TerminalView::spawn_shell("shell", 40, h.saturating_sub(2)) {
            desktop.insert_view(SlotId::Right, "shell", Box::new(term));
        }

        let status_bar = StatusBarView::new(StatusBarView::default_bindings());

        Self {
            desktop,
            status_bar,
            screen: Screen::new(w, h),
            running: true,
            keymap: KeymapKind::Vim,
            tree_outbox,
        }
    }

    /// Run the event loop.
    pub fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, terminal::EnterAlternateScreen)?;
        crossterm::execute!(stdout, crossterm::cursor::Hide)?;

        let (w, h) = terminal::size().unwrap_or((80, 24));
        self.layout(w, h);

        let result = self.event_loop(&mut stdout);

        let _ = crossterm::execute!(stdout, crossterm::cursor::Show);
        let _ = crossterm::execute!(stdout, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();

        result
    }

    fn event_loop(&mut self, out: &mut impl Write) -> io::Result<()> {
        let mut tick: u64 = 0;

        while self.running {
            if event::poll(Duration::from_millis(50))? {
                let ct_ev = event::read()?;
                self.dispatch_crossterm_event(ct_ev);
            }

            // Dispatch tick to desktop (for PTY polling)
            self.desktop.handle(&Event::Tick);

            // Drain tree outbox
            self.drain_tree_outbox();

            // Process status bar outbox
            self.process_status_outbox();

            tick += 1;
            self.draw(tick);
            self.screen.flush(out)?;
        }
        Ok(())
    }

    fn dispatch_crossterm_event(&mut self, ct_ev: CtEvent) {
        match ct_ev {
            CtEvent::Key(key) => {
                let ev = Event::Key(key);
                // Status bar sees keys first (preprocess)
                if self.status_bar.handle(&ev) == HandleResult::Consumed {
                    self.process_status_outbox();
                    return;
                }
                // Then desktop
                self.desktop.handle(&ev);
                // Check tree outbox after event dispatch
                self.drain_tree_outbox();
            }
            CtEvent::Resize(w, h) => {
                self.screen.resize(w, h);
                self.layout(w, h);
                self.desktop.handle(&Event::Resize(w, h));
            }
            _ => {}
        }
    }

    fn process_status_outbox(&mut self) {
        for cmd in self.status_bar.outbox.drain() {
            self.handle_command(cmd.id, cmd.payload);
        }
    }

    fn drain_tree_outbox(&mut self) {
        let commands: Vec<PendingCommand> = {
            if let Ok(mut outbox) = self.tree_outbox.lock() {
                outbox.drain()
            } else {
                Vec::new()
            }
        };
        for cmd in commands {
            self.handle_command(cmd.id, cmd.payload);
        }
    }

    fn handle_command(
        &mut self,
        cmd: u16,
        payload: Option<Box<dyn std::any::Any + Send>>,
    ) {
        match cmd {
            CM_QUIT => self.running = false,
            CM_OPEN_FILE => {
                if let Some(p) = payload {
                    if let Ok(ofp) = p.downcast::<OpenFilePayload>() {
                        self.open_file(Path::new(&ofp.path));
                    }
                }
            }
            CM_SHOW_HELP => self.show_help(),
            CM_NEW_SHELL => {
                let h = self.screen.height();
                if let Some(term) = TerminalView::spawn_shell("shell", 40, h.saturating_sub(2)) {
                    self.desktop.insert_view(SlotId::Right, "shell", Box::new(term));
                }
            }
            _ => {
                // Forward as a command event to the desktop
                self.desktop.handle(&Event::Command(cmd));
            }
        }
    }

    fn open_file(&mut self, path: &Path) {
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();

        // Check if already open in center slot — just switch to it
        if self.desktop.switch_to_tab(SlotId::Center, &title) {
            return;
        }

        // Open new editor tab
        if let Some(editor) = EditorView::open(path, self.keymap) {
            self.desktop.insert_view(SlotId::Center, &title, Box::new(editor));
        }
    }

    fn show_help(&mut self) {
        if self.desktop.switch_to_tab(SlotId::Center, "Help") {
            return;
        }

        let help_text = concat!(
            "kairn — TUI IDE\n\n",
            "F1: Help   F2: Tree   F3: Main   F4: Tools   F5: Zoom\n",
            "Ctrl-Q: Quit\n\n",
            "Navigation:\n",
            "  Enter/Right on file: open in center\n",
            "  Tab switching: Ctrl-Shift-Left/Right\n",
            "  Slot focus: F2-F4\n",
        );

        let buffer = crate::buffer::PieceTable::from_str(help_text);
        let km = self.keymap.create_keymap();
        let editor = crate::editor::Editor::new(buffer, km);
        let view = EditorView::from_editor(editor, "Help");
        self.desktop.insert_view(SlotId::Center, "Help", Box::new(view));
    }

    fn layout(&mut self, w: u16, h: u16) {
        let desktop_h = h.saturating_sub(1);
        self.desktop.set_bounds(Rect { x: 0, y: 0, w, h: desktop_h });
        self.status_bar.set_bounds(Rect { x: 0, y: desktop_h, w, h: 1 });
    }

    fn draw(&mut self, tick: u64) {
        let ctx = DrawContext { app_focused: true, tick };

        let desktop_bounds = self.desktop.bounds();
        {
            let mut surface = self.screen.surface(
                desktop_bounds.x, desktop_bounds.y,
                desktop_bounds.w, desktop_bounds.h,
            );
            self.desktop.draw(&mut surface, &ctx);
        }

        let sb_bounds = self.status_bar.bounds();
        {
            let mut surface = self.screen.surface(
                sb_bounds.x, sb_bounds.y, sb_bounds.w, sb_bounds.h,
            );
            self.status_bar.draw(&mut surface, &ctx);
        }
    }
}
