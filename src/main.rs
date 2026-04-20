// Deny unwrap/expect in non-test code — forces proper error handling.
#![deny(clippy::unwrap_used, clippy::expect_used)]
// Allow dead code during scaffold phase — remove once features are wired up.
#![allow(dead_code)]

mod app;
mod buffer;
mod capture;
mod cli;
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
    // Parse CLI args (handles -h/--help, -V/--version, exits on error)
    let cli = cli::Cli::parse_args();
    let project_path = cli.resolve_path();

    // Nesting guard
    if std::env::var("KAIRN_PID").is_ok() {
        eprintln!("kairn: already running (KAIRN_PID is set). Use the existing instance.");
        std::process::exit(1);
    }
    std::env::set_var("KAIRN_PID", std::process::id().to_string());

    install_panic_handler();

    let capture = capture::CapturePipe::create(&project_path).ok();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(
        project_path.to_string_lossy().to_string(),
        cli.config.as_deref(),
    );

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
        let area = Rect::new(0, 0, size.width, size.height.saturating_sub(1));
        let c = LayoutConstraints::compute(area, app.layout_mode, &app.panel_sizes);
        app.interactive.sync_size(c.interactive);
        // Main panel viewport height (inner area minus borders and gutter)
        app.main_view.viewport_h = c.main.height.saturating_sub(2) as usize;

        terminal.draw(|frame| {
            render_panels(frame, app);
            if let Some(search) = &app.search {
                render_search_overlay(frame, search);
            }
            if let Some(overlay) = &app.overlay {
                render_overlay(frame, overlay);
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
        if app.pending_redraw {
            app.pending_redraw = false;
            terminal.clear()?;
            continue;
        }

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }

        // Lazy load: expand file content when scrolling near bottom
        let near_bottom = app.main_view.scroll + 50 >= app.main_view.highlighted_lines.len();
        if near_bottom && app.main_view.current_path.is_some() {
            app.ensure_full_content();
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
    let full = frame.area();
    // Reserve bottom row for status bar
    let area = Rect::new(full.x, full.y, full.width, full.height.saturating_sub(1));
    let status_area = Rect::new(full.x, full.bottom().saturating_sub(1), full.width, 1);

    let c = LayoutConstraints::compute(area, app.layout_mode, &app.panel_sizes);

    if let Some(tree_rect) = c.tree {
        let focused = app.focus == panel::FocusedPanel::Tree;
        match app.left_mode {
            app::LeftPanelMode::FileTree => {
                app.file_tree.render(frame, tree_rect, focused);
            }
            app::LeftPanelMode::CommitTree => {
                app.commit_tree.render(frame, tree_rect, focused);
            }
        }
    }

    app.main_view
        .render(frame, c.main, app.focus == panel::FocusedPanel::Main);

    app.interactive.render(
        frame,
        c.interactive,
        app.focus == panel::FocusedPanel::Interactive,
    );

    render_status_bar(frame, app, status_area);
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

fn render_overlay(frame: &mut Frame, overlay: &Overlay) {
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

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let focus_name = match app.focus {
        panel::FocusedPanel::Tree => "Tree",
        panel::FocusedPanel::Main => "Main",
        panel::FocusedPanel::Interactive => "Terminal",
    };
    let tree_mode = match app.left_mode {
        app::LeftPanelMode::FileTree => {
            format!("Files:{}", app.file_tree.filter.label())
        }
        app::LeftPanelMode::CommitTree => "Commits".to_string(),
    };
    let main_mode = app.main_view.mode.label();
    let tab_name = app.interactive.tabs.active_title();

    let bg = Style::default().bg(Color::Black);
    let label = Style::default().fg(Color::LightCyan).bg(Color::Black);
    let value = Style::default().fg(Color::White).bg(Color::Black);

    let mut spans = vec![
        Span::styled(" [", label),
        Span::styled(focus_name, value),
        Span::styled("  Left:", label),
        Span::styled(&tree_mode, value),
        Span::styled("  Main:", label),
        Span::styled(main_mode, value),
        Span::styled("  Tab:", label),
        Span::styled(tab_name, value),
    ];

    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let right_content = "C-S-↑/↓:mode/tab  F3/4/5:panel  F1:help  Esc²:quit";
    let right_len = right_content.len() + 2; // +2 safety margin
    let pad = area
        .width
        .saturating_sub(left_len as u16 + right_len as u16);
    spans.push(Span::styled(" ".repeat(pad as usize), bg));
    spans.push(Span::styled("C-S-↑/↓:", label));
    spans.push(Span::styled("mode/tab  ", value));
    spans.push(Span::styled("F3/4/5:", label));
    spans.push(Span::styled("panel  ", value));
    spans.push(Span::styled("F1:", label));
    spans.push(Span::styled("help  ", value));
    spans.push(Span::styled("Esc²:", label));
    spans.push(Span::styled("quit", value));

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}
