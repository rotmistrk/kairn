// Deny unwrap/expect in non-test code — forces proper error handling.
#![deny(clippy::unwrap_used, clippy::expect_used)]
// Allow dead code during scaffold phase — remove once features are wired up.
#![allow(dead_code)]

mod app;
mod buffer;
mod capture;
mod config;
mod diff;
mod editor;
mod highlight;
mod keymap;
mod layout;
mod overlay;
mod panel;
mod search;
mod session;
mod tab;
mod termbuf;
mod tree;

use std::io;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};

use app::App;
use layout::LayoutConstraints;
use overlay::Overlay;
use panel::Panel;
use search::FileSearch;

fn main() -> Result<()> {
    // Nesting guard: prevent running kairn inside kairn
    if std::env::var("KAIRN_PID").is_ok() {
        eprintln!("kairn: already running (KAIRN_PID is set). Use the existing instance.");
        std::process::exit(1);
    }
    std::env::set_var("KAIRN_PID", std::process::id().to_string());

    install_panic_handler();

    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());

    // Create capture pipe before entering TUI
    let capture = capture::CapturePipe::create(&cwd).ok();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(cwd.to_string_lossy().to_string());

    // Init panel sizes from actual terminal dimensions
    let ts = terminal.size()?;
    app.init_panel_size(ts.width, ts.height);

    let result = run_loop(&mut terminal, &mut app, capture);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut capture: Option<capture::CapturePipe>,
) -> Result<()> {
    loop {
        // Sync PTY size to match panel dimensions
        let size = terminal.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        let c = LayoutConstraints::compute(area, app.layout_mode, &app.panel_sizes);
        app.interactive.sync_size(c.interactive);

        terminal.draw(|frame| {
            render_panels(frame, app);
            if let Some(search) = &app.search {
                render_search_overlay(frame, search);
            }
            if let Some(overlay) = &app.overlay {
                render_overlay(frame, overlay, &app.config);
            }
        })?;

        if app.should_quit {
            app.auto_save();
            return Ok(());
        }

        if app.pending_editor.is_some() {
            app.run_pending_editor()?;
            terminal.clear()?;
            continue;
        }
        if app.pending_shell {
            app.run_pending_shell()?;
            terminal.clear()?;
            continue;
        }
        if app.pending_peek {
            app.pending_peek = false;
            peek_screen()?;
            terminal.clear()?;
            continue;
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }

        app.interactive.tabs.poll_output();
        poll_capture(app, &mut capture);
    }
}

fn poll_capture(app: &mut App, capture: &mut Option<capture::CapturePipe>) {
    if let Some(ref mut pipe) = capture {
        if let Some(text) = pipe.poll() {
            app.show_captured(&text);
        }
    }
}

fn render_panels(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let c = LayoutConstraints::compute(area, app.layout_mode, &app.panel_sizes);

    if let Some(tree_rect) = c.tree {
        app.file_tree
            .render(frame, tree_rect, app.focus == panel::FocusedPanel::Tree);
    }

    app.main_view
        .render(frame, c.main, app.focus == panel::FocusedPanel::Main);

    app.interactive.render(
        frame,
        c.interactive,
        app.focus == panel::FocusedPanel::Interactive,
    );
}

fn render_search_overlay(frame: &mut Frame, search: &FileSearch) {
    let area = frame.area();
    let w = (area.width * 3 / 5).max(40).min(area.width);
    let h = 22u16.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + 2;
    let overlay = Rect::new(x, y, w, h);

    frame.render_widget(Clear, overlay);
    render_search_input(frame, search, overlay);
    render_search_results(frame, search, overlay);
}

fn render_search_input(frame: &mut Frame, search: &FileSearch, overlay: Rect) {
    let input_area = Rect::new(overlay.x, overlay.y, overlay.width, 3);
    let block = Block::default()
        .title(" 🔍 Search Files (Esc to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let para = Paragraph::new(search.query.as_str()).block(block);
    frame.render_widget(para, input_area);
    let cx = (input_area.x + 1 + search.cursor as u16).min(input_area.right().saturating_sub(2));
    let cy = input_area.y + 1;
    if cy < input_area.bottom().saturating_sub(1) {
        frame.set_cursor_position((cx, cy));
    }
}

fn render_search_results(frame: &mut Frame, search: &FileSearch, overlay: Rect) {
    let results_y = overlay.y + 3;
    let results_h = overlay.height.saturating_sub(3);
    let results_area = Rect::new(overlay.x, results_y, overlay.width, results_h);

    let items: Vec<ListItem<'_>> = search
        .results
        .iter()
        .enumerate()
        .take(results_h as usize)
        .map(|(i, r)| search_result_item(search, i, r))
        .collect();

    let count = search.results.len();
    let block = Block::default()
        .title(format!(" {count} results "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(List::new(items).block(block), results_area);
}

fn search_result_item<'a>(
    search: &'a FileSearch,
    index: usize,
    result: &search::SearchResult,
) -> ListItem<'a> {
    let path = search.result_path(result);
    let style = if index == search.selected {
        Style::default().bg(Color::Blue)
    } else {
        Style::default().fg(Color::White)
    };
    let score_str = if result.score > 0 {
        format!(" ({}) ", result.score)
    } else {
        String::new()
    };
    ListItem::new(Line::from(vec![
        Span::styled(format!(" {path}"), style),
        Span::styled(score_str, style.fg(Color::DarkGray)),
    ]))
}

fn render_overlay(frame: &mut Frame, overlay: &Overlay, config: &crate::config::Config) {
    let area = frame.area();
    let w = 50u16.min(area.width);
    let x = area.x + (area.width.saturating_sub(w)) / 2;

    match overlay {
        Overlay::SavePrompt(p) => {
            render_save_prompt(frame, p, Rect::new(x, area.y + 3, w, 3));
        }
        Overlay::LoadPicker(p) => {
            let h = (p.sessions.len() as u16 + 2).min(15).min(area.height);
            render_load_picker(frame, p, Rect::new(x, area.y + 3, w, h));
        }
        Overlay::Help => {
            render_help(frame, area, config);
        }
    }
}

fn render_save_prompt(frame: &mut Frame, prompt: &overlay::SavePrompt, area: Rect) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Save Session (Enter to save, Esc to cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let para = Paragraph::new(prompt.name.as_str()).block(block);
    frame.render_widget(para, area);
    let cx = (area.x + 1 + prompt.cursor as u16).min(area.right().saturating_sub(2));
    let cy = area.y + 1;
    if cy < area.bottom().saturating_sub(1) {
        frame.set_cursor_position((cx, cy));
    }
}

fn render_load_picker(frame: &mut Frame, picker: &overlay::LoadPicker, area: Rect) {
    frame.render_widget(Clear, area);
    let items: Vec<ListItem<'_>> = picker
        .sessions
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if i == picker.selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(format!(" {name}"), style))
        })
        .collect();
    let block = Block::default()
        .title(" Load Session (Enter to load, Esc to cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(List::new(items).block(block), area);
}

fn render_help(frame: &mut Frame, area: Rect, config: &crate::config::Config) {
    let text = build_help_text(config);
    let lines = text.lines().count() as u16 + 2;
    let w = 46u16.min(area.width);
    let h = lines.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let rect = Rect::new(x, area.y + 1, w, h);

    frame.render_widget(Clear, rect);
    let block = Block::default()
        .title(" Keybindings (any key to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(Paragraph::new(text).block(block), rect);
}

fn build_help_text(cfg: &crate::config::Config) -> String {
    let k = |name: &str| cfg.display_key(name);
    let bindings = [
        (k("quit"), "Quit"),
        ("Esc Esc".into(), "Quit (fallback)"),
        (k("rotate_layout"), "Rotate layout"),
        (k("toggle_tree"), "Toggle file tree"),
        (k("cycle_focus"), "Cycle panel focus"),
        (k("open_search"), "Fuzzy file search"),
        (k("diff_current_file"), "Diff file vs HEAD"),
        (k("git_log"), "Git commit log"),
        (k("launch_editor"), "Open in $EDITOR"),
        (k("new_kiro_tab"), "New Kiro tab"),
        (k("new_shell_tab"), "New shell tab"),
        (k("close_tab"), "Close tab"),
        (k("prev_tab"), "Previous tab"),
        (k("next_tab"), "Next tab"),
        (k("save_session"), "Save session"),
        (k("load_session"), "Load session"),
        (k("show_help"), "This help"),
    ];
    let mut text = String::new();
    for (key, desc) in &bindings {
        text.push_str(&format!(" {key:<16}{desc}\n"));
    }
    text.push('\n');
    text.push_str(" Main panel (focused):\n");
    text.push_str(" ↑/↓/PgUp/PgDn   Scroll\n");
    text.push_str(" Enter            Send to Kiro\n");
    text.push_str(" Alt-Enter        Send to shell\n");
    text.push('\n');
    text.push_str(" File tree (focused):\n");
    text.push_str(" j/k ↑/↓          Navigate\n");
    text.push_str(" Enter/l/→        Open / expand\n");
    text.push_str(" h/←              Collapse\n");
    text.push('\n');
    text.push_str(&format!(
        " Config: {}\n",
        crate::config::Config::global_rc().display()
    ));
    text
}

fn install_panic_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Restore terminal so error is visible
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);

        // Try to preserve state
        if let Ok(cwd) = std::env::current_dir() {
            let path = cwd.join(".kairn.panic.state");
            let _ = std::fs::copy(cwd.join(".kairn.state"), &path);
            eprintln!("kairn: state saved to {}", path.display());
        }

        eprintln!("kairn: unexpected panic — this is a bug.");
        eprintln!("kairn: {info}");
        eprintln!("kairn: please report at https://github.com/rotmistrk/kairn/issues");
        default_hook(info);
    }));
}

/// Midnight Commander-style peek: leave alternate screen, wait for any key, return.
fn peek_screen() -> Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    // Show hint
    eprintln!("-- kairn: press any key to return --");
    enable_raw_mode()?;
    // Wait for any key
    loop {
        if let Event::Key(_) = event::read()? {
            break;
        }
    }
    execute!(io::stdout(), EnterAlternateScreen)?;
    Ok(())
}
