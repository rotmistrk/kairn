//! Top-level application: owns all panels, drives the event loop.

use std::path::{Path, PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use txv::cell::{Color, Style};
use txv_widgets::{EventResult, LoopControl, RunContext, StatusSpan, Widget, WidgetAction};

use crate::config::Config;
use crate::editor::command::{EditorAction, EditorMode};
use crate::panel::bottom_panel::{BottomPanel, BottomTab};
use crate::panel::editor_panel::EditorPanel;
use crate::panel::status::AppStatusBar;
use crate::panel::terminal_panel::{PtyPoller, TerminalPanel};
use crate::panel::tree_panel::TreePanel;
use crate::panel::{LayoutState, PanelFocus, PanelRects, TriptychFocus};

/// Top-level application state.
pub struct App {
    workspace: PathBuf,
    layout: LayoutState,
    editor_panel: EditorPanel,
    bottom_panel: BottomPanel,
    status_bar: AppStatusBar,
    focus: PanelFocus,
    quit: bool,
    pending_chord: Option<KeyEvent>,
    config: Config,
    /// PTY pollers: index matches bottom panel tab index.
    pollers: Vec<Option<PtyPoller>>,
}

impl App {
    /// Create a new App for the given workspace.
    pub fn new(workspace: PathBuf, config_override: Option<&Path>) -> Self {
        let config = Config::load_with_override(&workspace, config_override);

        let tree = TreePanel::new(&workspace);
        let editor_panel = EditorPanel::new(tree);
        let bottom_panel = BottomPanel::new();
        let status_bar = AppStatusBar::new();

        Self {
            workspace,
            layout: LayoutState::default(),
            editor_panel,
            bottom_panel,
            status_bar,
            focus: PanelFocus::Editor,
            quit: false,
            pending_chord: None,
            config,
            pollers: Vec::new(),
        }
    }

    /// Main tick: called once per event loop iteration.
    pub fn tick(&mut self, ctx: &mut RunContext<'_>) -> LoopControl {
        self.handle_events(&ctx.events);
        self.poll_pty_data();

        let w = ctx.screen.width();
        let h = ctx.screen.height();
        self.layout.resize(w, h);

        self.update_status_bar();
        self.render(ctx);

        if self.quit {
            LoopControl::Quit
        } else {
            LoopControl::Continue
        }
    }

    // ── Event handling ──────────────────────────────────────────

    fn handle_events(&mut self, events: &[Event]) {
        for event in events {
            match event {
                Event::Key(key) => self.handle_key(*key),
                Event::Resize(w, h) => {
                    self.layout.resize(*w, *h);
                }
                _ => {}
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Handle pending chord.
        if let Some(prefix) = self.pending_chord.take() {
            self.handle_chord(prefix, key);
            return;
        }

        // Global keys first.
        if self.handle_global_key(key) {
            return;
        }

        // Dispatch to focused panel.
        let result = match self.focus {
            PanelFocus::Prompt => self.status_bar.handle_key(key),
            PanelFocus::Bottom => self.bottom_panel.handle_key(key),
            PanelFocus::Editor => self.editor_panel.handle_key(key),
        };

        self.handle_event_result(result);
    }

    /// Returns true if the key was consumed as a global binding.
    fn handle_global_key(&mut self, key: KeyEvent) -> bool {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        match key.code {
            // Ctrl-Q: quit
            KeyCode::Char('q') if ctrl => {
                self.quit = true;
                true
            }
            // Ctrl-L: cycle layout
            KeyCode::Char('l') if ctrl => {
                self.layout.cycle_mode();
                true
            }
            // Ctrl-B: toggle tree
            KeyCode::Char('b') if ctrl => {
                self.editor_panel.toggle_tree();
                self.layout.tree_visible = self.editor_panel.tree_visible;
                true
            }
            // Ctrl-X: start chord
            KeyCode::Char('x') if ctrl => {
                self.pending_chord = Some(key);
                true
            }
            // Alt-Left: previous bottom tab
            KeyCode::Left if alt => {
                self.bottom_panel.prev_tab();
                true
            }
            // Alt-Right: next bottom tab
            KeyCode::Right if alt => {
                self.bottom_panel.next_tab();
                true
            }
            // Ctrl-Shift-Up: cycle bottom tabs backward
            KeyCode::Up if ctrl && shift => {
                self.bottom_panel.prev_tab();
                true
            }
            // Ctrl-Shift-Down: cycle bottom tabs forward
            KeyCode::Down if ctrl && shift => {
                self.bottom_panel.next_tab();
                true
            }
            // F7: shrink tree
            KeyCode::F(7) => {
                let delta = if shift { -5 } else { -1 };
                self.layout.resize_tree(delta);
                true
            }
            // F8: grow tree
            KeyCode::F(8) => {
                let delta = if shift { 5 } else { 1 };
                self.layout.resize_tree(delta);
                true
            }
            // F9: shrink bottom
            KeyCode::F(9) => {
                self.layout.resize_bottom(-2);
                true
            }
            // F10: grow bottom
            KeyCode::F(10) => {
                self.layout.resize_bottom(2);
                true
            }
            // F2: cycle focus
            KeyCode::F(2) => {
                self.cycle_focus();
                true
            }
            // F3: focus tree
            KeyCode::F(3) => {
                if self.editor_panel.tree_visible {
                    self.focus = PanelFocus::Editor;
                    self.editor_panel.focus = TriptychFocus::Tree;
                }
                true
            }
            // F4: focus editor
            KeyCode::F(4) => {
                self.focus = PanelFocus::Editor;
                self.editor_panel.focus = TriptychFocus::Editor;
                true
            }
            // F5: focus terminal
            KeyCode::F(5) => {
                if self.bottom_panel.visible {
                    self.focus = PanelFocus::Bottom;
                }
                true
            }
            _ => false,
        }
    }

    /// Handle a two-key chord (Ctrl-X followed by another key).
    fn handle_chord(&mut self, _prefix: KeyEvent, key: KeyEvent) {
        match key.code {
            // Ctrl-X ↓: focus bottom
            KeyCode::Down => {
                if self.bottom_panel.visible {
                    self.focus = PanelFocus::Bottom;
                }
            }
            // Ctrl-X ↑: focus editor
            KeyCode::Up => {
                self.focus = PanelFocus::Editor;
            }
            // Ctrl-X →: cycle triptych right
            KeyCode::Right => {
                self.editor_panel.cycle_focus_right();
            }
            // Ctrl-X ←: cycle triptych left
            KeyCode::Left => {
                self.editor_panel.cycle_focus_left();
            }
            // Ctrl-X P: toggle control panel
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.editor_panel.toggle_control();
                self.layout.control_visible = self.editor_panel.control_visible;
            }
            // Ctrl-X \: toggle bottom panel
            KeyCode::Char('\\') => {
                self.bottom_panel.toggle();
                self.layout.bottom_visible = self.bottom_panel.visible;
                if !self.bottom_panel.visible && self.focus == PanelFocus::Bottom {
                    self.focus = PanelFocus::Editor;
                }
            }
            // Ctrl-X T: new shell tab
            KeyCode::Char('t') | KeyCode::Char('T') => {
                self.spawn_shell_tab();
            }
            // Ctrl-X N: new kiro tab
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.spawn_kiro_tab();
            }
            // Ctrl-X K: close tab
            KeyCode::Char('k') | KeyCode::Char('K') => {
                self.bottom_panel.close_active_tab();
            }
            _ => {} // Unknown chord — ignore.
        }
    }

    fn handle_event_result(&mut self, result: EventResult) {
        match result {
            EventResult::Action(WidgetAction::Selected(path)) => {
                // File selected from tree. PathBuf Debug format
                // wraps in quotes: "\"path\"".
                let clean = path.trim_matches('"').to_string();
                if std::path::Path::new(&clean).is_file() {
                    let _ = self.editor_panel.open_file(&clean);
                    self.editor_panel.focus = TriptychFocus::Editor;
                }
            }
            EventResult::Action(WidgetAction::Confirmed(text)) => {
                // Prompt confirmed.
                self.handle_prompt_confirmed(&text);
            }
            EventResult::Action(WidgetAction::Cancelled) => {
                if self.focus == PanelFocus::Prompt {
                    self.status_bar.dismiss_prompt();
                    self.focus = PanelFocus::Editor;
                }
            }
            EventResult::Action(WidgetAction::Close) => {
                // Editor wants to close buffer.
            }
            EventResult::Action(WidgetAction::Custom(action)) => {
                self.handle_custom_action(action);
            }
            _ => {}
        }
    }

    /// Cycle focus: Editor → Bottom → Editor.
    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            PanelFocus::Editor => {
                if self.bottom_panel.visible {
                    PanelFocus::Bottom
                } else {
                    PanelFocus::Editor
                }
            }
            PanelFocus::Bottom | PanelFocus::Prompt => PanelFocus::Editor,
        };
    }

    fn handle_prompt_confirmed(&mut self, text: &str) {
        self.status_bar.dismiss_prompt();
        self.focus = PanelFocus::Editor;

        // Handle ex-commands.
        if let Some(cmd) = text.strip_prefix(':') {
            let ex_cmd = crate::editor::ex::parse_ex(cmd);
            let _ = self.editor_panel.editor.execute(ex_cmd);
        }
    }

    fn handle_custom_action(&mut self, action: Box<dyn std::any::Any + Send>) {
        if let Ok(ea) = action.downcast::<EditorAction>() {
            match *ea {
                EditorAction::SaveRequested => {
                    self.save_current_file();
                }
                EditorAction::OpenFile(path) => {
                    let _ = self.editor_panel.open_file(&path);
                }
                _ => {}
            }
        }
    }

    fn save_current_file(&mut self) {
        let editor = self.editor_panel.editor.editor();
        let path = editor.buffer().file_path().map(|s| s.to_string());
        let content = editor.buffer().content();
        if let Some(path) = path {
            if crate::editor::save::atomic_save(&path, &content).is_ok() {
                self.editor_panel
                    .editor
                    .editor_mut()
                    .buffer_mut()
                    .mark_saved();
            }
        }
    }

    // ── PTY polling ─────────────────────────────────────────────

    fn poll_pty_data(&mut self) {
        use txv_widgets::Pollable;
        for (idx, poller) in self.pollers.iter_mut().enumerate() {
            if let Some(ref mut p) = poller {
                if let Some(data) = p.poll() {
                    self.bottom_panel.process_poll_data(idx, &data);
                }
            }
        }
    }

    // ── Terminal spawning ───────────────────────────────────────

    fn spawn_shell_tab(&mut self) {
        let rects = self.layout.compute_rects();
        let (cols, rows) = bottom_size(&rects);
        let mut term = TerminalPanel::new("shell", cols, rows);
        if let Ok(poller) = term.spawn_shell() {
            self.pollers.push(Some(poller));
            self.bottom_panel.add_terminal(BottomTab::Terminal(term));
            self.bottom_panel.visible = true;
            self.layout.bottom_visible = true;
        }
    }

    fn spawn_kiro_tab(&mut self) {
        let rects = self.layout.compute_rects();
        let (cols, rows) = bottom_size(&rects);
        let kiro_cmd = self.config.kiro_command.clone();
        let mut term = TerminalPanel::new("kiro", cols, rows);
        if let Ok(poller) = term.spawn_kiro(&kiro_cmd) {
            self.pollers.push(Some(poller));
            self.bottom_panel.add_terminal(BottomTab::Kiro(term));
            self.bottom_panel.visible = true;
            self.layout.bottom_visible = true;
        }
    }

    // ── Status bar ──────────────────────────────────────────────

    fn update_status_bar(&mut self) {
        let editor = self.editor_panel.editor.editor();
        let mode = editor.mode();
        let (line, col) = editor.cursor();
        let modified = editor.buffer().is_modified();

        let mode_style = Style {
            fg: Color::Rgb(235, 219, 178),
            bg: Color::Palette(239),
            ..Style::default()
        };

        let mode_label = match mode {
            EditorMode::Normal => "vi:NORMAL",
            EditorMode::Insert => "vi:INSERT",
            EditorMode::Visual(_) => "vi:VISUAL",
            EditorMode::CommandLine => "vi:COMMAND",
        };

        let mut left = vec![StatusSpan {
            text: format!(" {mode_label}"),
            style: mode_style,
        }];

        if modified {
            left.push(StatusSpan {
                text: " [+]".into(),
                style: Style {
                    fg: Color::Rgb(250, 189, 47),
                    bg: Color::Palette(239),
                    ..Style::default()
                },
            });
        }

        if let Some(path) = editor.buffer().file_path() {
            left.push(StatusSpan {
                text: format!("  {path}"),
                style: mode_style,
            });
        }

        left.push(StatusSpan {
            text: format!("  {}:{}", line + 1, col + 1),
            style: mode_style,
        });

        let right = vec![StatusSpan {
            text: " F1:help  Ctrl-Q:quit ".into(),
            style: mode_style,
        }];

        self.status_bar.set_left(left);
        self.status_bar.set_right(right);
    }

    // ── Rendering ───────────────────────────────────────────────

    fn render(&self, ctx: &mut RunContext<'_>) {
        let rects = self.layout.compute_rects();

        // Tree panel.
        if let Some(tree_rect) = rects.tree {
            if tree_rect.w > 0 && tree_rect.h > 0 {
                let mut s = ctx
                    .screen
                    .surface(tree_rect.x, tree_rect.y, tree_rect.w, tree_rect.h);
                let focused = self.focus == PanelFocus::Editor
                    && self.editor_panel.focus == TriptychFocus::Tree;
                self.editor_panel.tree.render(&mut s, focused);
            }
        }

        // Editor.
        {
            let r = rects.editor;
            if r.w > 0 && r.h > 0 {
                let mut s = ctx.screen.surface(r.x, r.y, r.w, r.h);
                let focused = self.focus == PanelFocus::Editor
                    && self.editor_panel.focus == TriptychFocus::Editor;
                self.editor_panel.editor.render(&mut s, focused);
            }
        }

        // Control panel.
        if let Some(ctrl_rect) = rects.control {
            if ctrl_rect.w > 0 && ctrl_rect.h > 0 {
                let mut s = ctx
                    .screen
                    .surface(ctrl_rect.x, ctrl_rect.y, ctrl_rect.w, ctrl_rect.h);
                let focused = self.focus == PanelFocus::Editor
                    && self.editor_panel.focus == TriptychFocus::Control;
                self.editor_panel.control.render(&mut s, focused);
            }
        }

        // Bottom panel.
        if let Some(bot_rect) = rects.bottom {
            if bot_rect.w > 0 && bot_rect.h > 0 {
                let mut s = ctx
                    .screen
                    .surface(bot_rect.x, bot_rect.y, bot_rect.w, bot_rect.h);
                self.bottom_panel
                    .render(&mut s, self.focus == PanelFocus::Bottom);
            }
        }

        // Status bar.
        {
            let r = rects.status;
            if r.w > 0 && r.h > 0 {
                let mut s = ctx.screen.surface(r.x, r.y, r.w, r.h);
                self.status_bar
                    .render(&mut s, self.focus == PanelFocus::Prompt);
            }
        }
    }
}

/// Extract bottom panel dimensions from rects.
fn bottom_size(rects: &PanelRects) -> (u16, u16) {
    rects
        .bottom
        .map(|r| (r.w, r.h.saturating_sub(1)))
        .unwrap_or((80, 24))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn test_app() -> App {
        let workspace = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        App::new(workspace, None)
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    // ── Focus tests ─────────────────────────────────────────────

    #[test]
    fn initial_focus_is_editor() {
        let app = test_app();
        assert_eq!(app.focus, PanelFocus::Editor);
    }

    #[test]
    fn ctrl_q_sets_quit() {
        let mut app = test_app();
        app.handle_key(ctrl('q'));
        assert!(app.quit);
    }

    #[test]
    fn ctrl_l_cycles_layout() {
        let mut app = test_app();
        let initial = app.layout.mode;
        app.handle_key(ctrl('l'));
        assert_ne!(app.layout.mode, initial);
    }

    #[test]
    fn ctrl_b_toggles_tree() {
        let mut app = test_app();
        assert!(app.editor_panel.tree_visible);
        app.handle_key(ctrl('b'));
        assert!(!app.editor_panel.tree_visible);
        assert!(!app.layout.tree_visible);
        app.handle_key(ctrl('b'));
        assert!(app.editor_panel.tree_visible);
    }

    // ── Chord tests ─────────────────────────────────────────────

    #[test]
    fn ctrl_x_starts_chord() {
        let mut app = test_app();
        app.handle_key(ctrl('x'));
        assert!(app.pending_chord.is_some());
    }

    #[test]
    fn ctrl_x_down_focuses_bottom() {
        let mut app = test_app();
        app.bottom_panel.visible = true;
        app.layout.bottom_visible = true;
        app.handle_key(ctrl('x'));
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.focus, PanelFocus::Bottom);
    }

    #[test]
    fn ctrl_x_up_focuses_editor() {
        let mut app = test_app();
        app.focus = PanelFocus::Bottom;
        app.handle_key(ctrl('x'));
        app.handle_key(key(KeyCode::Up));
        assert_eq!(app.focus, PanelFocus::Editor);
    }

    #[test]
    fn ctrl_x_p_toggles_control() {
        let mut app = test_app();
        assert!(!app.editor_panel.control_visible);
        app.handle_key(ctrl('x'));
        app.handle_key(key(KeyCode::Char('p')));
        assert!(app.editor_panel.control_visible);
    }

    #[test]
    fn ctrl_x_backslash_toggles_bottom() {
        let mut app = test_app();
        assert!(app.bottom_panel.visible);
        app.handle_key(ctrl('x'));
        app.handle_key(key(KeyCode::Char('\\')));
        assert!(!app.bottom_panel.visible);
    }

    #[test]
    fn unknown_chord_clears_pending() {
        let mut app = test_app();
        app.handle_key(ctrl('x'));
        assert!(app.pending_chord.is_some());
        app.handle_key(key(KeyCode::Char('z'))); // unknown
        assert!(app.pending_chord.is_none());
    }

    // ── Focus cycling tests ─────────────────────────────────────

    #[test]
    fn f2_cycles_focus() {
        let mut app = test_app();
        app.bottom_panel.visible = true;
        app.layout.bottom_visible = true;
        assert_eq!(app.focus, PanelFocus::Editor);
        app.handle_key(key(KeyCode::F(2)));
        assert_eq!(app.focus, PanelFocus::Bottom);
        app.handle_key(key(KeyCode::F(2)));
        assert_eq!(app.focus, PanelFocus::Editor);
    }

    #[test]
    fn f3_focuses_tree() {
        let mut app = test_app();
        app.handle_key(key(KeyCode::F(3)));
        assert_eq!(app.focus, PanelFocus::Editor);
        assert_eq!(app.editor_panel.focus, TriptychFocus::Tree);
    }

    #[test]
    fn f4_focuses_editor() {
        let mut app = test_app();
        app.editor_panel.focus = TriptychFocus::Tree;
        app.handle_key(key(KeyCode::F(4)));
        assert_eq!(app.editor_panel.focus, TriptychFocus::Editor);
    }

    #[test]
    fn f5_focuses_terminal() {
        let mut app = test_app();
        app.bottom_panel.visible = true;
        app.layout.bottom_visible = true;
        app.handle_key(key(KeyCode::F(5)));
        assert_eq!(app.focus, PanelFocus::Bottom);
    }

    // ── Resize tests ────────────────────────────────────────────

    #[test]
    fn f7_shrinks_tree() {
        let mut app = test_app();
        let before = app.layout.tree_width;
        app.handle_key(key(KeyCode::F(7)));
        assert_eq!(app.layout.tree_width, before - 1);
    }

    #[test]
    fn f8_grows_tree() {
        let mut app = test_app();
        let before = app.layout.tree_width;
        app.handle_key(key(KeyCode::F(8)));
        assert_eq!(app.layout.tree_width, before + 1);
    }

    #[test]
    fn shift_f7_shrinks_tree_by_5() {
        let mut app = test_app();
        let before = app.layout.tree_width;
        let shift_f7 = KeyEvent::new(KeyCode::F(7), KeyModifiers::SHIFT);
        app.handle_key(shift_f7);
        assert_eq!(app.layout.tree_width, before - 5);
    }

    // ── Triptych focus tests ────────────────────────────────────

    #[test]
    fn ctrl_x_right_cycles_triptych() {
        let mut app = test_app();
        app.editor_panel.focus = TriptychFocus::Tree;
        app.handle_key(ctrl('x'));
        app.handle_key(key(KeyCode::Right));
        assert_eq!(app.editor_panel.focus, TriptychFocus::Editor);
    }

    #[test]
    fn ctrl_x_left_cycles_triptych_back() {
        let mut app = test_app();
        app.editor_panel.focus = TriptychFocus::Editor;
        app.handle_key(ctrl('x'));
        app.handle_key(key(KeyCode::Left));
        assert_eq!(app.editor_panel.focus, TriptychFocus::Tree);
    }

    // ── Hide focused panel moves focus ──────────────────────────

    #[test]
    fn hide_tree_moves_focus_to_editor() {
        let mut app = test_app();
        app.editor_panel.focus = TriptychFocus::Tree;
        app.handle_key(ctrl('b')); // toggle tree off
        assert_eq!(app.editor_panel.focus, TriptychFocus::Editor);
    }

    #[test]
    fn hide_bottom_moves_focus_to_editor() {
        let mut app = test_app();
        app.focus = PanelFocus::Bottom;
        app.bottom_panel.visible = true;
        app.handle_key(ctrl('x'));
        app.handle_key(key(KeyCode::Char('\\'))); // toggle bottom off
        assert_eq!(app.focus, PanelFocus::Editor);
    }
}
