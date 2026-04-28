use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::config::Config;
use crate::editor;
use crate::highlight::Highlighter;
use crate::keymap::{Action, Keymap};
use crate::layout::{LayoutMode, PanelSizes};
use crate::overlay::{LoadPicker, Overlay, OverlayAction, SavePrompt};
use crate::panel::file_tree::FileTreePanel;
use crate::panel::interactive::InteractivePanel;
use crate::panel::main_view::MainViewPanel;
use crate::panel::{FocusedPanel, Panel, PanelAction};
use crate::search::{FileSearch, SearchAction};
use crate::session::{self, Session};

/// Which panel is shown on the left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LeftPanelMode {
    #[default]
    FileTree,
    CommitTree,
}

/// Top-level application state.
pub struct App {
    pub workspace_root: PathBuf,
    pub layout_mode: LayoutMode,
    pub panel_sizes: PanelSizes,
    pub focus: FocusedPanel,
    pub left_mode: LeftPanelMode,
    pub file_tree: FileTreePanel,
    pub commit_tree: crate::panel::commit_tree::CommitTreePanel,
    pub main_view: MainViewPanel,
    pub interactive: InteractivePanel,
    pub should_quit: bool,
    pub pending_editor: Option<String>,
    pub pending_shell: bool,
    pub pending_peek: bool,
    pub pending_redraw: bool,
    pub highlighter: Highlighter,
    pub config: Config,
    pub keymap: Keymap,
    pub search: Option<FileSearch>,
    pub overlay: Option<Overlay>,
    /// Cache: path → (mtime_secs, content, highlighted lines)
    file_cache: std::collections::HashMap<String, (u64, String, Vec<ratatui::text::Line<'static>>)>,
}

impl App {
    pub fn new(workspace_root: String, config_override: Option<&std::path::Path>) -> Self {
        let ws = PathBuf::from(&workspace_root);
        let config = Config::load_with_override(&ws, config_override);
        let keymap = Keymap::from_config(&config);
        let mut app = Self {
            workspace_root: ws.clone(),
            layout_mode: LayoutMode::default(),
            panel_sizes: PanelSizes::default(),
            focus: FocusedPanel::default(),
            left_mode: LeftPanelMode::default(),
            file_tree: FileTreePanel::new(workspace_root),
            commit_tree: crate::panel::commit_tree::CommitTreePanel::new(ws.clone()),
            main_view: MainViewPanel::default(),
            interactive: InteractivePanel::default(),
            should_quit: false,
            pending_editor: None,
            pending_shell: false,
            pending_peek: false,
            pending_redraw: false,
            highlighter: Highlighter::new(),
            config,
            keymap,
            search: None,
            overlay: None,
            file_cache: std::collections::HashMap::new(),
        };
        app.main_view.line_numbers = app.config.line_numbers;
        app.main_view.tab_width = app.config.tab_width;
        // Try auto-restore, otherwise show welcome
        if !app.try_auto_restore() {
            app.show_welcome();
        }
        app
    }

    /// Call once with terminal size before creating tabs.
    pub fn init_panel_size(&mut self, width: u16, height: u16) {
        let area = ratatui::layout::Rect::new(0, 0, width, height);
        let c =
            crate::layout::LayoutConstraints::compute(area, self.layout_mode, &self.panel_sizes);
        self.interactive.sync_size(c.interactive);
    }

    fn try_auto_restore(&mut self) -> bool {
        let sess = match session::auto_load(&self.workspace_root) {
            Ok(Some(s)) => s,
            _ => return false,
        };
        self.restore_session(sess);
        true
    }

    /// Spawn live PTYs for restored tabs that have none.
    pub fn revive_tabs(&mut self) {
        let (cols, rows) = self.interactive.inner_size();
        self.interactive.tabs.revive_tabs(
            &self.config.kiro_command,
            cols,
            rows,
            &self.workspace_root,
        );
    }

    fn restore_session(&mut self, sess: Session) {
        self.layout_mode = sess.layout_mode;
        self.panel_sizes = sess.panel_sizes;
        self.interactive.tabs.restore(sess.tabs, sess.active_tab);
        if let Some(path) = sess.open_file {
            self.open_file(&path);
        }
    }

    /// Snapshot current state into a Session.
    pub fn snapshot_session(&self, name: &str) -> Session {
        let (tabs, active_tab) = self.interactive.tabs.snapshot();
        Session {
            name: name.to_string(),
            workspace_root: self.workspace_root.to_string_lossy().to_string(),
            layout_mode: self.layout_mode,
            panel_sizes: self.panel_sizes.clone(),
            tabs,
            active_tab,
            open_file: self.main_view.current_file_path().map(String::from),
        }
    }

    /// Auto-save state to $PWD/.kairn.state.
    pub fn auto_save(&mut self) {
        // Capture kiro-cli session IDs so we can resume them on restore.
        let ids = session::list_kiro_sessions(&self.config.kiro_command);
        self.interactive.tabs.stamp_kiro_sessions(&ids);
        let sess = self.snapshot_session("_auto");
        let _ = session::auto_save(&self.workspace_root, &sess);
    }

    /// Display captured output from KAIRN_CAPTURE pipe in main panel.
    fn cycle_mode(&mut self, forward: bool) {
        match self.focus {
            FocusedPanel::Tree => {
                if forward {
                    self.file_tree.filter = self.file_tree.filter.next();
                } else {
                    // Reverse: cycle 3 times forward
                    self.file_tree.filter = self.file_tree.filter.next().next();
                }
                self.file_tree.cursor = 0;
                self.file_tree.git_status =
                    crate::panel::file_tree::collect_git_status(&self.file_tree.root_path);
            }
            FocusedPanel::Main => {
                self.main_view.mode = if forward {
                    self.main_view.mode.next()
                } else {
                    self.main_view.mode.next().next().next()
                };
                self.apply_view_mode();
            }
            FocusedPanel::Interactive => {
                if forward {
                    self.interactive.tabs.next_tab();
                } else {
                    self.interactive.tabs.prev_tab();
                }
            }
        }
    }

    fn scroll_focused(&mut self, delta: isize) {
        match self.focus {
            FocusedPanel::Main => {
                self.main_view.scroll_by(delta, 20);
            }
            FocusedPanel::Interactive => {
                if let Some(tb) = self.interactive.tabs.active_termbuf_mut() {
                    if delta < 0 {
                        tb.scroll_up((-delta) as usize);
                    } else {
                        tb.scroll_down(delta as usize);
                    }
                }
            }
            FocusedPanel::Tree => {}
        }
    }

    /// Display captured output from KAIRN_CAPTURE pipe in main panel.
    pub fn show_captured(&mut self, text: &str) {
        let buf = crate::buffer::OutputBuffer::plain("captured".to_string(), text.to_string());
        self.main_view.set_buffer(buf);
    }

    /// Scrape the active terminal tab's full content into the main panel.
    fn capture_all(&mut self) {
        let title = format!("{} [all]", self.interactive.tabs.active_title());
        let text = match self.interactive.tabs.active_termbuf() {
            Some(tb) => crate::termbuf::extract_text(tb),
            None => return,
        };
        let buf = crate::buffer::OutputBuffer::plain(title, text);
        self.main_view.set_buffer(buf);
        self.focus = FocusedPanel::Main;
    }

    /// Capture last output (since last prompt) from the active terminal.
    fn capture_output(&mut self) {
        let title = format!("{} [output]", self.interactive.tabs.active_title());
        let text = match self.interactive.tabs.active_termbuf() {
            Some(tb) => crate::termbuf::extract_last_output(tb),
            None => return,
        };
        let buf = crate::buffer::OutputBuffer::plain(title, text);
        self.main_view.set_buffer(buf);
        self.focus = FocusedPanel::Main;
    }

    /// Open a prompt to save the current main panel buffer to a file.
    fn open_save_buffer(&mut self) {
        let default = self
            .main_view
            .buffer
            .as_ref()
            .map(|b| b.title.clone())
            .unwrap_or_default();
        let suggested = if default.contains('[') {
            // For captured buffers, suggest a .txt filename
            default
                .split_whitespace()
                .next()
                .unwrap_or("output")
                .to_string()
                + ".txt"
        } else {
            default
        };
        self.overlay = Some(Overlay::SaveFilePrompt(
            crate::overlay::SaveFilePrompt::new(&suggested),
        ));
    }

    /// Write the current main panel buffer content to a file.
    fn save_buffer_to_file(&mut self, path: &str) {
        let content = self
            .main_view
            .buffer
            .as_ref()
            .map(|b| b.content.as_str())
            .unwrap_or("");
        if let Err(e) = std::fs::write(path, content) {
            eprintln!("kairn: write failed: {e}");
        }
    }

    fn show_welcome(&mut self) {
        let buf = crate::buffer::OutputBuffer::plain("kairn".to_string(), String::new());
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(crate::styled::welcome_lines(&self.config));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.overlay.is_some() {
            self.keymap.cancel_pending();
            return self.handle_overlay_key(key);
        }
        if self.search.is_some() {
            self.keymap.cancel_pending();
            return self.handle_search_key(key);
        }
        let result = self.keymap.map_key(key);
        let action = match result {
            crate::keymap::MapResult::Pending => return Ok(()),
            crate::keymap::MapResult::Action(a) => a,
        };
        // When the terminal panel is focused, only intercept global
        // bindings (F-keys, Ctrl+Shift, Ctrl+Q). Forward everything
        // else so readline / shell editing keys reach the PTY.
        if self.focus == FocusedPanel::Interactive {
            if let Action::Forward(_) = action {
                // not mapped — forward as usual
            } else if action.is_global() {
                // global binding — handle at app level
            } else {
                // app binding that conflicts with terminal input
                return self.forward_to_panel(key);
            }
        }
        self.dispatch_action(action)
    }

    fn dispatch_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::RotateLayout => {
                self.layout_mode = self.layout_mode.next();
                self.pending_redraw = true;
            }
            Action::ToggleTree => self.panel_sizes.toggle_tree(),
            Action::CycleFocus => self.focus = self.focus.next(),
            Action::FocusTree => self.focus = FocusedPanel::Tree,
            Action::FocusMain => self.focus = FocusedPanel::Main,
            Action::FocusTerminal => self.focus = FocusedPanel::Interactive,
            Action::ResizeTree(d) => {
                if self.focus == FocusedPanel::Interactive
                    && self.layout_mode != LayoutMode::Wide
                {
                    // In stacked layouts, F7/F8 resize the interactive panel vertically
                    self.panel_sizes.resize_interactive_height(-d);
                } else {
                    self.panel_sizes.resize_tree(d);
                }
            }
            Action::ResizeInteractive(d) => {
                if self.layout_mode == LayoutMode::Wide {
                    self.panel_sizes.resize_interactive_width(d);
                } else {
                    self.panel_sizes.resize_interactive_height(d);
                }
            }
            Action::PeekScreen => self.pending_peek = true,
            Action::Redraw => self.pending_redraw = true,
            Action::RefreshTree => self.file_tree.refresh(),
            Action::SuspendToShell => self.pending_shell = true,
            Action::LaunchEditor => {
                self.pending_editor = self.main_view.current_file_path().map(String::from);
            }
            Action::ToggleLeftPanel => self.toggle_left(),
            Action::ScrollUp => self.scroll_focused(-20),
            Action::ScrollDown => self.scroll_focused(20),
            Action::ScrollTop => self.scroll_focused(-100_000),
            Action::ScrollBottom => self.scroll_focused(100_000),
            Action::CycleModeNext => self.cycle_mode(true),
            Action::CycleModePrev => self.cycle_mode(false),
            Action::OpenSearch => self.open_search(),
            Action::DiffCurrentFile => self.diff_current_file(),
            Action::GitLog => self.show_git_log(),
            Action::SaveSession => self.open_save_prompt(),
            Action::LoadSession => self.open_load_picker(),
            Action::ShowHelp => self.show_help(),
            Action::CaptureAll => self.capture_all(),
            Action::CaptureOutput => self.capture_output(),
            Action::SaveBuffer => self.open_save_buffer(),
            Action::Forward(key) => self.forward_to_panel(key)?,
            action => self.handle_tab_action(action),
        }
        Ok(())
    }

    fn toggle_left(&mut self) {
        self.left_mode = match self.left_mode {
            LeftPanelMode::FileTree => LeftPanelMode::CommitTree,
            LeftPanelMode::CommitTree => LeftPanelMode::FileTree,
        };
    }

    fn open_save_prompt(&mut self) {
        self.overlay = Some(Overlay::SavePrompt(SavePrompt::new()));
    }

    fn open_load_picker(&mut self) {
        let names = session::list_sessions().unwrap_or_default();
        if names.is_empty() {
            return;
        }
        self.overlay = Some(Overlay::LoadPicker(LoadPicker::new(names)));
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> Result<()> {
        let action = match &mut self.overlay {
            Some(Overlay::SavePrompt(p)) => p.handle_key(key),
            Some(Overlay::SaveFilePrompt(p)) => p.handle_key(key),
            Some(Overlay::LoadPicker(p)) => p.handle_key(key),
            None => return Ok(()),
        };
        match action {
            OverlayAction::None => {}
            OverlayAction::Close => self.overlay = None,
            OverlayAction::Save(name) => {
                self.overlay = None;
                self.save_session(&name);
            }
            OverlayAction::SaveFile(path) => {
                self.overlay = None;
                self.save_buffer_to_file(&path);
            }
            OverlayAction::Load(name) => {
                self.overlay = None;
                self.load_session(&name);
            }
        }
        Ok(())
    }

    fn save_session(&mut self, name: &str) {
        let ids = session::list_kiro_sessions(&self.config.kiro_command);
        self.interactive.tabs.stamp_kiro_sessions(&ids);
        let sess = self.snapshot_session(name);
        let _ = session::save(&sess);
    }

    fn load_session(&mut self, name: &str) {
        let sess = match session::load(name) {
            Ok(s) => s,
            Err(_) => return,
        };
        self.restore_session(sess);
    }

    fn open_search(&mut self) {
        let mut search = FileSearch::new(&self.workspace_root);
        search.update_results();
        self.search = Some(search);
    }

    fn diff_current_file(&mut self) {
        let path = match self.main_view.current_file_path() {
            Some(p) => p.to_string(),
            None => return,
        };
        let diff_lines = match crate::diff::diff_vs_head(Path::new(&path)) {
            Ok(Some(lines)) => lines,
            _ => return,
        };
        let styled = crate::styled::diff_lines_to_styled(&diff_lines);
        let raw = diff_lines
            .iter()
            .map(|l| l.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let buf = crate::buffer::OutputBuffer::diff(format!("diff: {path}"), raw);
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(styled);
    }

    fn show_git_log(&mut self) {
        let file_filter = self.main_view.current_file_path().and_then(|p| {
            // Convert to relative path for gix
            Path::new(p)
                .strip_prefix(&self.workspace_root)
                .ok()
                .map(|r| r.to_string_lossy().to_string())
        });
        let entries = match crate::diff::git_log(&self.workspace_root, file_filter.as_deref(), 200)
        {
            Ok(e) => e,
            Err(_) => return,
        };
        let (styled, raw) = crate::styled::log_entries_to_styled(&entries);
        let title = match &file_filter {
            Some(f) => format!("log: {f}"),
            None => "log: (all)".to_string(),
        };
        let buf = crate::buffer::OutputBuffer::plain(title, raw);
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(styled);
    }

    fn expand_macros(&self, text: &str) -> String {
        let mut out = text.to_string();
        if let Some(path) = &self.main_view.current_path {
            out = out.replace("@file", path);
            let name = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            out = out.replace("@name", &name);
        }
        out = out.replace("@dir", &self.workspace_root.to_string_lossy());
        let line = self.main_view.cursor.0 + 1;
        out = out.replace("@line", &line.to_string());
        out
    }

    fn expand_and_send_line(&mut self) {
        let line = match self.interactive.tabs.active_termbuf() {
            Some(tb) => {
                let (row, _) = tb.cursor();
                let cells = tb.visible_row(row);
                cells.iter().map(|c| c.ch).collect::<String>()
            }
            None => return,
        };
        let trimmed = line.trim_end();
        // Clear current line (Ctrl-U), send expanded text + Enter
        self.interactive.tabs.write_to_active(b"\x15");
        let expanded = self.expand_macros(trimmed);
        self.interactive.tabs.write_to_active(expanded.as_bytes());
        self.interactive.tabs.write_to_active(b"\r");
    }

    fn show_commit_diff(&mut self, hash: &str) {
        let output = std::process::Command::new("git")
            .args(["show", "--stat", "--patch", hash])
            .current_dir(&self.workspace_root)
            .env("TERM", "dumb")
            .output();
        let text = match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(e) => format!("Error: {e}"),
        };
        let lines = self.highlight_to_owned(&text, "commit.diff");
        let buf = crate::buffer::OutputBuffer::plain(format!("commit {hash}"), text);
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(lines);
    }

    fn show_help(&mut self) {
        let text = crate::help::build_full_help(&self.config);
        let lines = self.highlight_to_owned(&text, "help.md");
        let buf = crate::buffer::OutputBuffer::plain("kairn help".to_string(), text);
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(lines);
        self.main_view.current_path = None;
        self.focus = FocusedPanel::Main;
    }

    fn show_blame(&mut self) {
        let path = match &self.main_view.current_path {
            Some(p) => p.clone(),
            None => return,
        };
        let blame_lines = match crate::diff::git_blame(Path::new(&path)) {
            Ok(l) => l,
            Err(_) => return,
        };
        let (styled, raw) = crate::styled::blame_to_styled(&blame_lines);
        let buf = crate::buffer::OutputBuffer::plain(format!("blame: {path}"), raw);
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(styled);
    }

    fn show_table(&mut self) {
        let path = match &self.main_view.current_path {
            Some(p) => p.clone(),
            None => return,
        };
        let (styled, raw) = crate::csv_table::csv_to_table(&path);
        let buf = crate::buffer::OutputBuffer::plain(format!("table: {path}"), raw);
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(styled);
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        let action = match &mut self.search {
            Some(search) => search.handle_key(key),
            None => return Ok(()),
        };
        match action {
            SearchAction::None => {}
            SearchAction::Close => self.search = None,
            SearchAction::Open(rel_path) => {
                self.search = None;
                let full = self.workspace_root.join(&rel_path);
                self.open_file(&full.to_string_lossy());
            }
        }
        Ok(())
    }

    fn handle_tab_action(&mut self, action: Action) {
        match action {
            Action::NewKiroTab => {
                let (cols, rows) = self.interactive.inner_size();
                self.interactive.tabs.add_kiro_tab(
                    &self.config.kiro_command,
                    cols,
                    rows,
                    &self.workspace_root,
                );
                self.focus = FocusedPanel::Interactive;
            }
            Action::NewShellTab => {
                let (cols, rows) = self.interactive.inner_size();
                self.interactive
                    .tabs
                    .add_shell_tab(cols, rows, &self.workspace_root);
                self.focus = FocusedPanel::Interactive;
            }
            Action::NextTab => self.interactive.tabs.next_tab(),
            Action::PrevTab => self.interactive.tabs.prev_tab(),
            Action::CloseTab => self.interactive.tabs.close_active(),
            _ => {}
        }
    }

    fn forward_to_panel(&mut self, key: KeyEvent) -> Result<()> {
        let panel_action = match self.focus {
            FocusedPanel::Tree => match self.left_mode {
                LeftPanelMode::FileTree => self.file_tree.handle_key(key)?,
                LeftPanelMode::CommitTree => self.commit_tree.handle_key(key)?,
            },
            FocusedPanel::Main => self.main_view.handle_key(key)?,
            FocusedPanel::Interactive => self.interactive.handle_key(key)?,
        };
        self.process_panel_action(panel_action);
        Ok(())
    }

    pub fn run_pending_editor(&mut self) -> Result<()> {
        if let Some(path) = self.pending_editor.take() {
            editor::launch_editor(&path)?;
        }
        Ok(())
    }

    pub fn run_pending_shell(&mut self) -> Result<()> {
        if self.pending_shell {
            self.pending_shell = false;
            self.auto_save();
            editor::launch_shell()?;
        }
        Ok(())
    }

    fn process_panel_action(&mut self, action: PanelAction) {
        match action {
            PanelAction::None => {}
            PanelAction::OpenFile(path) => self.open_file(&path),
            PanelAction::PreviewFile(path) => {
                self.main_view.current_path = Some(path);
                self.apply_view_mode();
            }
            PanelAction::SwitchMode => self.apply_view_mode(),
            PanelAction::SendToKiro(text) => {
                let expanded = self.expand_macros(&text);
                self.interactive.tabs.write_to_active(expanded.as_bytes());
            }
            PanelAction::PreviewCommit(hash) => {
                self.show_commit_diff(&hash);
            }
            PanelAction::ExpandLine => {
                self.expand_and_send_line();
            }
            PanelAction::Yank(text) => {
                crate::styled::osc52_copy(&text);
            }
            PanelAction::FocusRight => {
                self.focus = match self.focus {
                    FocusedPanel::Tree => FocusedPanel::Main,
                    FocusedPanel::Main => FocusedPanel::Interactive,
                    FocusedPanel::Interactive => FocusedPanel::Interactive,
                };
            }
            PanelAction::FocusLeft => {
                self.focus = match self.focus {
                    FocusedPanel::Tree => FocusedPanel::Tree,
                    FocusedPanel::Main => FocusedPanel::Tree,
                    FocusedPanel::Interactive => FocusedPanel::Main,
                };
            }
            PanelAction::PushOutput(buf) => {
                self.main_view.set_buffer(buf);
            }
            PanelAction::Quit => self.should_quit = true,
        }
    }

    fn apply_view_mode(&mut self) {
        use crate::panel::main_view::ViewMode;
        let path = match &self.main_view.current_path {
            Some(p) => p.clone(),
            None => return,
        };
        match self.main_view.mode {
            ViewMode::File => self.open_file(&path),
            ViewMode::Diff => self.diff_current_file(),
            ViewMode::Log => self.show_git_log(),
            ViewMode::Blame => self.show_blame(),
            ViewMode::Table => self.show_table(),
        }
    }

    /// Reload the currently viewed file if its mtime changed on disk.
    pub fn reload_if_changed(&mut self) {
        let path = match self.main_view.current_file_path() {
            Some(p) => p.to_string(),
            None => return,
        };
        let disk_mtime = file_mtime(&path);
        let cached_mtime = self.file_cache.get(&path).map(|c| c.0).unwrap_or(0);
        if disk_mtime != cached_mtime {
            let scroll = self.main_view.scroll;
            self.open_file(&path);
            self.main_view.scroll = scroll;
        }
    }

    pub fn open_file(&mut self, path: &str) {
        let mtime = file_mtime(path);
        let (content, owned_lines) = if let Some(cached) = self.file_cache.get(path) {
            if cached.0 == mtime {
                (cached.1.clone(), cached.2.clone())
            } else {
                self.read_and_cache(path, mtime)
            }
        } else {
            self.read_and_cache(path, mtime)
        };
        let buf = crate::buffer::OutputBuffer {
            title: path.to_string(),
            content,
            kind: crate::buffer::BufferKind::FilePreview {
                path: path.to_string(),
            },
            scroll_offset: 0,
        };
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(owned_lines);
    }

    fn read_and_cache(
        &mut self,
        path: &str,
        mtime: u64,
    ) -> (String, Vec<ratatui::text::Line<'static>>) {
        let raw =
            std::fs::read_to_string(path).unwrap_or_else(|e| format!("Error reading {path}: {e}"));
        // Expand tabs to spaces
        let tab_str = " ".repeat(self.main_view.tab_width.max(1));
        let full = raw.replace('\t', &tab_str);
        let total_lines = full.lines().count();
        // Lazy: only highlight first 5000 lines initially
        let limit = 5000;
        let c = if total_lines > limit {
            let truncated: String = full.lines().take(limit).collect::<Vec<_>>().join("\n");
            format!(
                "{truncated}\n\n... ({} more lines, scroll down to load) ...",
                total_lines - limit
            )
        } else {
            full.clone()
        };
        let lines = self.highlight_to_owned(&c, path);
        // Cache full content for later loading
        self.file_cache
            .insert(path.to_string(), (mtime, full, lines.clone()));
        (c, lines)
    }

    /// Load full file content if we were showing truncated version.
    pub fn ensure_full_content(&mut self) {
        let path = match &self.main_view.current_path {
            Some(p) => p.clone(),
            None => return,
        };
        if let Some(cached) = self.file_cache.get(&path) {
            let full = &cached.1;
            let current_lines = self.main_view.highlighted_lines.len();
            let full_lines = full.lines().count();
            if current_lines < full_lines {
                let lines = self.highlight_to_owned(full, &path);
                let buf = crate::buffer::OutputBuffer {
                    title: path.clone(),
                    content: full.clone(),
                    kind: crate::buffer::BufferKind::FilePreview { path },
                    scroll_offset: 0,
                };
                let scroll = self.main_view.scroll;
                self.main_view.set_buffer(buf);
                self.main_view.set_highlighted(lines);
                self.main_view.scroll = scroll;
            }
        }
    }

    fn highlight_to_owned(&self, content: &str, path: &str) -> Vec<ratatui::text::Line<'static>> {
        self.highlighter
            .highlight_file(content, path)
            .into_iter()
            .map(|line| {
                let spans: Vec<ratatui::text::Span<'static>> = line
                    .spans
                    .into_iter()
                    .map(|s| {
                        ratatui::text::Span::styled(
                            s.content.trim_end_matches(&['\n', '\r'][..]).to_string(),
                            s.style,
                        )
                    })
                    .collect();
                ratatui::text::Line::from(spans)
            })
            .collect()
    }
}


fn file_mtime(path: &str) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}




