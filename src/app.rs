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
    pub pending_shell: bool,
    pub pending_peek: bool,
    pub highlighter: Highlighter,
    pub config: Config,
    pub keymap: Keymap,
    pub search: Option<FileSearch>,
    pub overlay: Option<Overlay>,
    last_esc: bool,
    /// Cache: path → (mtime_secs, content, highlighted lines)
    file_cache: std::collections::HashMap<String, (u64, String, Vec<ratatui::text::Line<'static>>)>,
}

impl App {
    pub fn new(workspace_root: String) -> Self {
        let ws = PathBuf::from(&workspace_root);
        let config = Config::load(&ws);
        let keymap = Keymap::from_config(&config);
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
            pending_shell: false,
            pending_peek: false,
            highlighter: Highlighter::new(),
            config,
            keymap,
            search: None,
            overlay: None,
            last_esc: false,
            file_cache: std::collections::HashMap::new(),
        };
        app.main_view.line_numbers = app.config.line_numbers;
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

    fn show_welcome(&mut self) {
        let buf = crate::buffer::OutputBuffer::plain("kairn".to_string(), String::new());
        self.main_view.set_buffer(buf);
        self.main_view.set_highlighted(welcome_lines(&self.config));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Double-Esc to quit (works even if Ctrl-Q is eaten by terminal)
        if key.code == crossterm::event::KeyCode::Esc {
            if self.overlay.is_some() || self.search.is_some() {
                // First Esc closes overlay/search, reset
                self.last_esc = false;
            } else if self.last_esc {
                self.should_quit = true;
                return Ok(());
            } else {
                self.last_esc = true;
                return Ok(());
            }
        } else {
            self.last_esc = false;
        }

        if self.overlay.is_some() {
            return self.handle_overlay_key(key);
        }
        if self.search.is_some() {
            return self.handle_search_key(key);
        }
        self.dispatch_action(self.keymap.map_key(key))
    }

    fn dispatch_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.should_quit = true,
            Action::RotateLayout => {
                self.layout_mode = self.layout_mode.next();
            }
            Action::ToggleTree => self.panel_sizes.toggle_tree(),
            Action::CycleFocus => self.focus = self.focus.next(),
            Action::FocusTree => self.focus = FocusedPanel::Tree,
            Action::FocusMain => self.focus = FocusedPanel::Main,
            Action::FocusTerminal => self.focus = FocusedPanel::Interactive,
            Action::ResizeTree(d) => self.panel_sizes.resize_tree(d),
            Action::ResizeInteractive(d) => {
                self.panel_sizes.resize_interactive(d);
            }
            Action::TogglePinOutput => {}
            Action::PeekScreen => {
                self.pending_peek = true;
            }
            Action::ScrollUp => self.scroll_focused(-20),
            Action::ScrollDown => self.scroll_focused(20),
            Action::ScrollTop => self.scroll_focused(-100_000),
            Action::ScrollBottom => self.scroll_focused(100_000),
            Action::CycleModeNext => self.cycle_mode(true),
            Action::CycleModePrev => self.cycle_mode(false),
            Action::LaunchEditor => {
                self.pending_editor = self.main_view.current_file_path().map(String::from);
            }
            Action::SuspendToShell => {
                self.pending_shell = true;
            }
            Action::OpenSearch => self.open_search(),
            Action::DiffCurrentFile => self.diff_current_file(),
            Action::GitLog => self.show_git_log(),
            Action::SaveSession => self.open_save_prompt(),
            Action::LoadSession => self.open_load_picker(),
            Action::ShowHelp => self.show_help(),
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

    fn show_help(&mut self) {
        let text = build_full_help(&self.config);
        let buf = crate::buffer::OutputBuffer::plain("kairn help".to_string(), text);
        self.main_view.set_buffer(buf);
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
        let (styled, raw) = blame_to_styled(&blame_lines);
        let buf = crate::buffer::OutputBuffer::plain(format!("blame: {path}"), raw);
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
                    "new",
                    &self.config.kiro_command,
                    cols,
                    rows,
                    &self.workspace_root,
                );
            }
            Action::NewShellTab => {
                let (cols, rows) = self.interactive.inner_size();
                self.interactive
                    .tabs
                    .add_shell_tab(cols, rows, &self.workspace_root);
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
                // Write selected text to active kiro/shell tab's PTY
                self.interactive.tabs.write_to_active(text.as_bytes());
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
        let c =
            std::fs::read_to_string(path).unwrap_or_else(|e| format!("Error reading {path}: {e}"));
        let lines = self.highlight_to_owned(&c, path);
        self.file_cache
            .insert(path.to_string(), (mtime, c.clone(), lines.clone()));
        (c, lines)
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

fn welcome_lines(cfg: &Config) -> Vec<ratatui::text::Line<'static>> {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let dim = Style::default().fg(Color::DarkGray);
    let mut lines = welcome_banner();
    lines.extend(welcome_keys(cfg));

    for warn in cfg.detect_collisions() {
        lines.push(Line::from(Span::styled(
            format!("  {warn}"),
            Style::default().fg(Color::Red),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  Config: {}", Config::global_rc().display()),
        dim,
    )));
    lines
}

fn welcome_banner() -> Vec<ratatui::text::Line<'static>> {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};

    let bold = Style::default().add_modifier(Modifier::BOLD);
    let cyan = bold.fg(Color::Cyan);
    let dim = Style::default().fg(Color::DarkGray);
    let white = Style::default().fg(Color::White);

    vec![
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
        Line::from(Span::styled(
            "  Named after cairn — stones marking a trail.",
            dim,
        )),
        Line::from(""),
    ]
}

fn welcome_keys(cfg: &Config) -> Vec<ratatui::text::Line<'static>> {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let y = Style::default().fg(Color::Yellow);
    let w = Style::default().fg(Color::White);
    let k = |name: &str| cfg.display_key(name);

    vec![
        Line::from(Span::styled("  Quick start:", y)),
        Line::from(Span::styled(
            format!("  {:<14} Search files", k("open_search")),
            w,
        )),
        Line::from(Span::styled(
            format!("  {:<14} Open shell tab", k("new_shell_tab")),
            w,
        )),
        Line::from(Span::styled(
            format!("  {:<14} Open Kiro tab", k("new_kiro_tab")),
            w,
        )),
        Line::from(Span::styled(
            format!("  {:<14} Diff vs HEAD", k("diff_current_file")),
            w,
        )),
        Line::from(Span::styled(format!("  {:<14} Git log", k("git_log")), w)),
        Line::from(Span::styled(
            format!("  {:<14} All keybindings", k("show_help")),
            w,
        )),
    ]
}

fn blame_to_styled(
    lines: &[crate::diff::BlameLine],
) -> (Vec<ratatui::text::Line<'static>>, String) {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let mut styled = Vec::new();
    let mut raw = String::new();

    for bl in lines {
        let line_str = format!(
            "{} {:>12} {} │ {}",
            bl.hash_short, bl.author, bl.date, bl.content
        );
        raw.push_str(&line_str);
        raw.push('\n');

        styled.push(Line::from(vec![
            Span::styled(
                format!("{} ", bl.hash_short),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("{:>12} ", bl.author),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("{} │ ", bl.date),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(bl.content.clone(), Style::default().fg(Color::White)),
        ]));
    }

    if lines.is_empty() {
        styled.push(Line::from("(no blame data)"));
        raw.push_str("(no blame data)\n");
    }

    (styled, raw)
}

fn file_mtime(path: &str) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn build_full_help(cfg: &Config) -> String {
    let k = |name: &str| cfg.display_key(name);
    let ks = |name: &str| {
        let src = cfg.key_source(name);
        format!("{:<18} {:<14} ({})", k(name), name, src.label())
    };
    let mut h = String::new();

    // Logo
    h.push('\n');
    h.push_str("  ╦╔═╔═╗╦╦═╗╔╗╔\n");
    h.push_str("  ╠╩╗╠═╣║╠╦╝║║║   v0.1.0\n");
    h.push_str("  ╩ ╩╩ ╩╩╩╚═╝╚╝   A TUI IDE for Kiro AI\n");
    h.push('\n');
    h.push_str("  Named after cairn — stacked stones marking a trail.\n");
    h.push('\n');

    // Panel navigation
    h.push_str("═══ Panel Navigation ═══\n\n");
    h.push_str(&format!("  {}\n", ks("focus_tree")));
    h.push_str(&format!("  {}\n", ks("focus_main")));
    h.push_str(&format!("  {}\n", ks("focus_terminal")));
    h.push_str(&format!("  {}\n", ks("cycle_focus")));
    h.push_str(&format!("  {}\n", ks("rotate_layout")));
    h.push_str(&format!("  {}\n", ks("toggle_tree")));
    h.push_str(&format!("  {}\n", ks("cycle_mode_next")));
    h.push_str(&format!("  {}\n", ks("cycle_mode_prev")));
    h.push('\n');

    // File operations
    h.push_str("═══ File Operations ═══\n\n");
    h.push_str(&format!("  {}\n", ks("open_search")));
    h.push_str(&format!("  {}\n", ks("launch_editor")));
    h.push_str(&format!("  {}\n", ks("show_help")));
    h.push('\n');

    // Git
    h.push_str("═══ Git ═══\n\n");
    h.push_str(&format!("  {}\n", ks("diff_current_file")));
    h.push_str(&format!("  {}\n", ks("git_log")));
    h.push_str("  Mode cycle (main panel): File → Diff → Log → Blame\n");
    h.push_str("  Filter cycle (tree):     All → Modified → Untracked\n");
    h.push('\n');

    // Terminal tabs
    h.push_str("═══ Terminal Tabs ═══\n\n");
    h.push_str(&format!("  {}\n", ks("new_kiro_tab")));
    h.push_str(&format!("  {}\n", ks("new_shell_tab")));
    h.push_str(&format!("  {}\n", ks("close_tab")));
    h.push_str(&format!("  {}\n", ks("prev_tab")));
    h.push_str(&format!("  {}\n", ks("next_tab")));
    h.push_str("  PgUp/PgDn                                Scroll back\n");
    h.push('\n');

    // Session & system
    h.push_str("═══ Session & System ═══\n\n");
    h.push_str(&format!("  {}\n", ks("save_session")));
    h.push_str(&format!("  {}\n", ks("load_session")));
    h.push_str(&format!("  {}\n", ks("suspend_to_shell")));
    h.push_str(&format!("  {}\n", ks("peek_screen")));
    h.push_str(&format!("  {}\n", ks("quit")));
    h.push_str("  Esc Esc                                   Quit (fallback)\n");
    h.push('\n');

    // Main panel cursor mode
    h.push_str("═══ Main Panel ═══\n\n");
    h.push_str("  Scroll mode (default):\n");
    h.push_str("    ↑/↓/PgUp/PgDn     Scroll\n");
    h.push_str("    /                  Search (type to find, n/N next/prev)\n");
    h.push('\n');
    h.push_str("  Cursor mode (Space to toggle):\n");
    h.push_str("    ↑↓←→              Move cursor\n");
    h.push_str("    v                  Stream (character) select\n");
    h.push_str("    V                  Line select\n");
    h.push_str("    Ctrl-V             Block (column) select\n");
    h.push_str("    Enter              Send selection to terminal tab\n");
    h.push_str("    Esc                Clear selection\n");
    h.push_str("    /                  Search\n");
    h.push('\n');

    // File tree
    h.push_str("═══ File Tree ═══\n\n");
    h.push_str("  j/k ↑/↓              Navigate\n");
    h.push_str("  Enter/l/→            Open file / expand dir\n");
    h.push_str("  h/←                  Collapse dir\n");
    h.push_str("  Auto-preview: files show in main panel on cursor move\n");
    h.push_str("  Git colors: yellow=modified green=added red=deleted\n");
    h.push('\n');

    // Configuration
    h.push_str("═══ Configuration ═══\n\n");
    h.push_str(&format!("  Global:   {}\n", Config::global_rc().display()));
    h.push_str("  Project:  $PWD/.kairnrc (overrides global)\n");
    h.push_str("  State:    $PWD/.kairn.state (auto-saved on quit)\n");
    h.push_str("  Format:   JSON — only set what you want to change\n");
    h.push('\n');
    h.push_str("  Example .kairnrc:\n");
    h.push_str("  {\n");
    h.push_str("    \"kiro_command\": \"kiro-cli\",\n");
    h.push_str("    \"line_numbers\": true,\n");
    h.push_str("    \"keys\": {\n");
    h.push_str("      \"quit\": \"ctrl+q\",\n");
    h.push_str("      \"new_shell_tab\": \"ctrl+s\"\n");
    h.push_str("    }\n");
    h.push_str("  }\n");
    h.push('\n');

    // Environment variables
    h.push_str("═══ Environment Variables ═══\n\n");
    h.push_str("  KAIRN_PID       Set on start. Prevents nested kairn instances.\n");
    h.push_str("                  If set, kairn exits with a message.\n\n");
    h.push_str("  KAIRN_CAPTURE   Named pipe (FIFO) for output capture.\n");
    h.push_str("                  From a suspended shell (Ctrl-T):\n");
    h.push_str("                    $ ls -la > $KAIRN_CAPTURE\n");
    h.push_str("                    $ cat src/main.rs > $KAIRN_CAPTURE\n");
    h.push_str("                    $ cargo test 2>&1 > $KAIRN_CAPTURE\n");
    h.push_str("                  Output appears in main panel when you return.\n\n");
    h.push_str("  SHELL           Used for shell tabs and Ctrl-T suspend.\n");
    h.push_str("  EDITOR          Used for Ctrl-E (open file in editor).\n");
    h.push('\n');

    // Conflicts
    let conflicts = cfg.detect_collisions();
    if !conflicts.is_empty() {
        h.push_str("═══ ⚠ Key Conflicts ═══\n\n");
        for c in &conflicts {
            h.push_str(&format!("  {c}\n"));
        }
        h.push('\n');
    }

    // Full binding dump with sources
    h.push_str("═══ All Effective Keybindings ═══\n\n");
    h.push_str("  Key              Action                       Source\n");
    h.push_str("  ───              ──────                       ──────\n");
    let mut keys: Vec<_> = cfg.keys.iter().collect();
    keys.sort_by_key(|(k, _)| k.as_str());
    for (action, combo) in &keys {
        if combo.0.is_empty() {
            continue;
        }
        let src = cfg.key_source(action).label();
        h.push_str(&format!("  {:<18} {:<28} {}\n", combo.0, action, src));
    }
    h
}
