use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::editor;
use crate::highlight::Highlighter;
use crate::input::SendTarget;
use crate::keymap::{self, Action};
use crate::layout::{LayoutMode, PanelSizes};
use crate::overlay::{LoadPicker, Overlay, OverlayAction, SavePrompt};
use crate::panel::file_tree::FileTreePanel;
use crate::panel::interactive::InteractivePanel;
use crate::panel::main_view::MainViewPanel;
use crate::panel::{FocusedPanel, Panel, PanelAction};
use crate::search::{FileSearch, SearchAction};
use crate::session::{self, Session};

/// Top-level application state.
pub struct App {
    pub workspace_root: PathBuf,
    pub layout_mode: LayoutMode,
    pub panel_sizes: PanelSizes,
    pub focus: FocusedPanel,
    pub file_tree: FileTreePanel,
    pub main_view: MainViewPanel,
    pub interactive: InteractivePanel,
    pub should_quit: bool,
    pub pending_editor: Option<String>,
    pub highlighter: Highlighter,
    pub search: Option<FileSearch>,
    pub overlay: Option<Overlay>,
}

impl App {
    pub fn new(workspace_root: String) -> Self {
        Self {
            workspace_root: PathBuf::from(&workspace_root),
            layout_mode: LayoutMode::default(),
            panel_sizes: PanelSizes::default(),
            focus: FocusedPanel::default(),
            file_tree: FileTreePanel::new(workspace_root),
            main_view: MainViewPanel::default(),
            interactive: InteractivePanel::default(),
            should_quit: false,
            pending_editor: None,
            highlighter: Highlighter::new(),
            search: None,
            overlay: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.overlay.is_some() {
            return self.handle_overlay_key(key);
        }
        if self.search.is_some() {
            return self.handle_search_key(key);
        }
        self.dispatch_action(keymap::map_key(key))
    }

    fn dispatch_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::RotateLayout => {
                self.layout_mode = self.layout_mode.next();
            }
            Action::ToggleTree => self.panel_sizes.toggle_tree(),
            Action::CycleFocus => self.focus = self.focus.next(),
            Action::ResizeTree(d) => self.panel_sizes.resize_tree(d),
            Action::ResizeInteractive(d) => {
                self.panel_sizes.resize_interactive(d);
            }
            Action::TogglePinOutput => {}
            Action::LaunchEditor => {
                self.pending_editor = self.main_view.current_file_path().map(String::from);
            }
            Action::OpenSearch => self.open_search(),
            Action::DiffCurrentFile => self.diff_current_file(),
            Action::SaveSession => self.open_save_prompt(),
            Action::LoadSession => self.open_load_picker(),
            Action::Forward(key) => self.forward_to_panel(key)?,
            action => self.handle_tab_action(action),
        }
        Ok(())
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
            OverlayAction::Load(name) => {
                self.overlay = None;
                self.load_session(&name);
            }
        }
        Ok(())
    }

    fn save_session(&self, name: &str) {
        let (tabs, active_tab) = self.interactive.tabs.snapshot();
        let open_file = self.main_view.current_file_path().map(String::from);
        let sess = Session {
            name: name.to_string(),
            workspace_root: self.workspace_root.to_string_lossy().to_string(),
            layout_mode: self.layout_mode,
            panel_sizes: self.panel_sizes.clone(),
            tabs,
            active_tab,
            open_file,
        };
        let _ = session::save(&sess);
    }

    fn load_session(&mut self, name: &str) {
        let sess = match session::load(name) {
            Ok(s) => s,
            Err(_) => return,
        };
        self.layout_mode = sess.layout_mode;
        self.panel_sizes = sess.panel_sizes;
        self.interactive.tabs.restore(sess.tabs, sess.active_tab);
        if let Some(path) = sess.open_file {
            self.open_file(&path);
        }
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
        let styled = diff_lines_to_styled(&diff_lines);
        let raw = diff_lines
            .iter()
            .map(|l| l.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let buf = crate::buffer::OutputBuffer::diff(format!("diff: {path}"), raw);
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
                self.interactive.tabs.add_kiro_tab("new".into());
            }
            Action::NewShellTab => {
                self.interactive.tabs.add_shell_tab("bash".into());
            }
            Action::NextTab => self.interactive.tabs.next_tab(),
            Action::PrevTab => self.interactive.tabs.prev_tab(),
            Action::CloseTab => self.interactive.tabs.close_active(),
            _ => {}
        }
    }

    fn forward_to_panel(&mut self, key: KeyEvent) -> Result<()> {
        let panel_action = match self.focus {
            FocusedPanel::Tree => self.file_tree.handle_key(key)?,
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

    fn process_panel_action(&mut self, action: PanelAction) {
        match action {
            PanelAction::None => {}
            PanelAction::OpenFile(path) => self.open_file(&path),
            PanelAction::PushOutput(buf) => {
                self.main_view.set_buffer(buf);
            }
            PanelAction::SendInput { text, target } => {
                self.dispatch_input(&text, target);
            }
            PanelAction::Quit => self.should_quit = true,
        }
    }

    pub fn open_file(&mut self, path: &str) {
        let content =
            std::fs::read_to_string(path).unwrap_or_else(|e| format!("Error reading {path}: {e}"));
        let owned_lines = self.highlight_to_owned(&content, path);
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

    fn highlight_to_owned(&self, content: &str, path: &str) -> Vec<ratatui::text::Line<'static>> {
        self.highlighter
            .highlight_file(content, path)
            .into_iter()
            .map(|line| {
                let spans: Vec<ratatui::text::Span<'static>> = line
                    .spans
                    .into_iter()
                    .map(|s| ratatui::text::Span::styled(s.content.to_string(), s.style))
                    .collect();
                ratatui::text::Line::from(spans)
            })
            .collect()
    }

    fn dispatch_input(&mut self, text: &str, target: SendTarget) {
        let prefix = match target {
            SendTarget::Kiro => "→kiro",
            SendTarget::Terminal => "→shell",
        };
        self.interactive
            .tabs
            .push_to_active(format!("[{prefix}] {text}"));
    }
}

fn diff_lines_to_styled(lines: &[crate::diff::DiffLine]) -> Vec<ratatui::text::Line<'static>> {
    use crate::diff::DiffTag;
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};

    lines
        .iter()
        .map(|dl| {
            let style = match dl.tag {
                DiffTag::Header => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                DiffTag::Added => Style::default().fg(Color::Green),
                DiffTag::Removed => Style::default().fg(Color::Red),
                DiffTag::Context => Style::default().fg(Color::White),
            };
            Line::from(Span::styled(dl.content.clone(), style))
        })
        .collect()
}
