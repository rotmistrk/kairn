use std::path::{Path, PathBuf};

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::config::Config;
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
    pub config: Config,
    pub search: Option<FileSearch>,
    pub overlay: Option<Overlay>,
}

impl App {
    pub fn new(workspace_root: String) -> Self {
        let ws = PathBuf::from(&workspace_root);
        let config = Config::load(&ws);
        let mut app = Self {
            workspace_root: ws,
            layout_mode: LayoutMode::default(),
            panel_sizes: PanelSizes::default(),
            focus: FocusedPanel::default(),
            file_tree: FileTreePanel::new(workspace_root),
            main_view: MainViewPanel::default(),
            interactive: InteractivePanel::default(),
            should_quit: false,
            pending_editor: None,
            highlighter: Highlighter::new(),
            config,
            search: None,
            overlay: None,
        };
        // Try auto-restore, otherwise show welcome
        if !app.try_auto_restore() {
            app.show_welcome();
        }
        app
    }

    fn try_auto_restore(&mut self) -> bool {
        let sess = match session::auto_load(&self.workspace_root) {
            Ok(Some(s)) => s,
            _ => return false,
        };
        self.restore_session(sess);
        true
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
    pub fn auto_save(&self) {
        let sess = self.snapshot_session("_auto");
        let _ = session::auto_save(&self.workspace_root, &sess);
    }

    fn show_welcome(&mut self) {
        let buf = crate::buffer::OutputBuffer::plain("kairn".to_string(), String::new());
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(welcome_lines());
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
            Action::GitLog => self.show_git_log(),
            Action::SaveSession => self.open_save_prompt(),
            Action::LoadSession => self.open_load_picker(),
            Action::ShowHelp => {
                self.overlay = Some(Overlay::Help);
            }
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
            Some(Overlay::Help) => OverlayAction::Close,
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
        let (styled, raw) = log_entries_to_styled(&entries);
        let title = match &file_filter {
            Some(f) => format!("log: {f}"),
            None => "log: (all)".to_string(),
        };
        let buf = crate::buffer::OutputBuffer::plain(title, raw);
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

fn log_entries_to_styled(
    entries: &[crate::diff::LogEntry],
) -> (Vec<ratatui::text::Line<'static>>, String) {
    let mut lines = Vec::new();
    let mut raw = String::new();

    for e in entries {
        raw.push_str(&format!(
            "{} {} {} {}\n",
            e.hash_short, e.date, e.author, e.message
        ));
        lines.push(log_entry_line(e));
    }

    if entries.is_empty() {
        lines.push(ratatui::text::Line::from("(no commits found)"));
        raw.push_str("(no commits found)\n");
    }

    (lines, raw)
}

fn log_entry_line(e: &crate::diff::LogEntry) -> ratatui::text::Line<'static> {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::Span;

    ratatui::text::Line::from(vec![
        Span::styled(
            format!("{} ", e.hash_short),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{} ", e.date), Style::default().fg(Color::Cyan)),
        Span::styled(format!("{} ", e.author), Style::default().fg(Color::Green)),
        Span::styled(e.message.clone(), Style::default().fg(Color::White)),
    ])
}

fn welcome_lines() -> Vec<ratatui::text::Line<'static>> {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};

    let bold = Style::default().add_modifier(Modifier::BOLD);
    let cyan = bold.fg(Color::Cyan);
    let dim = Style::default().fg(Color::DarkGray);
    let white = Style::default().fg(Color::White);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("  ╦╔═╔═╗╦╦═╗╔╗╔", cyan)),
        Line::from(Span::styled("  ╠╩╗╠═╣║╠╦╝║║║", cyan)),
        Line::from(Span::styled("  ╩ ╩╩ ╩╩╩╚═╝╚╝", cyan)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  kairn", bold),
            Span::styled(" v0.1.0", dim),
        ]),
        Line::from(""),
        Line::from(Span::styled("  A TUI IDE oriented around Kiro AI.", white)),
        Line::from(""),
        Line::from(Span::styled(
            "  Named after cairn — stacked stones marking a trail.",
            dim,
        )),
        Line::from(Span::styled(
            "  Kairn guides your path through code, with AI at the core.",
            dim,
        )),
        Line::from(""),
    ];
    lines.extend(welcome_keys());
    lines
}

fn welcome_keys() -> Vec<ratatui::text::Line<'static>> {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let y = Style::default().fg(Color::Yellow);
    let w = Style::default().fg(Color::White);
    let d = Style::default().fg(Color::DarkGray);

    vec![
        Line::from(Span::styled("  Quick start:", y)),
        Line::from(Span::styled("  Ctrl-P       Search files", w)),
        Line::from(Span::styled("  Ctrl-S       Open shell tab", w)),
        Line::from(Span::styled("  Ctrl-K       Open Kiro tab", w)),
        Line::from(Span::styled("  Ctrl-D       Diff vs HEAD", w)),
        Line::from(Span::styled("  Ctrl-G       Git log", w)),
        Line::from(Span::styled("  Ctrl-/ / F1  All keybindings", w)),
        Line::from(""),
        Line::from(Span::styled(
            "  Navigate the file tree, or Ctrl-P to jump to a file.",
            d,
        )),
    ]
}
