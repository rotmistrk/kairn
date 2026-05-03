// Deny unwrap/expect in non-test code — forces proper error handling.
#![deny(clippy::unwrap_used, clippy::expect_used)]
// Allow dead code during scaffold phase — remove once features are wired up.
#![allow(dead_code)]

mod app;
mod buffer;
mod capture;
mod cli;
mod config;
mod content_search;
mod csv_table;
mod diff;
mod editor;
mod git;
mod help;
mod highlight;
mod keymap;
mod layout;
mod nav;
mod overlay;
mod panel;
mod rusticle_bridge;
mod search;
mod session;
mod styled;
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
    let cli = cli::Cli::parse_args();
    let project_path = cli.resolve_path();

    if std::env::var("KAIRN_PID").is_ok() {
        eprintln!("kairn: already running (KAIRN_PID is set). Use the existing instance.");
        std::process::exit(1);
    }
    std::env::set_var("KAIRN_PID", std::process::id().to_string());
    install_panic_handler();

    let capture = capture::CapturePipe::create(&project_path).ok();
    let mut terminal = init_terminal()?;

    let mut app = App::new(
        project_path.to_string_lossy().to_string(),
        cli.config.as_deref(),
    );
    let ts = terminal.size()?;
    app.init_panel_size(ts.width, ts.height);
    app.revive_tabs();

    let result = run_loop(&mut terminal, &mut app, capture);
    restore_terminal(&mut terminal)?;
    result
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableBracketedPaste
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableBracketedPaste
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    mut capture: Option<capture::CapturePipe>,
) -> Result<()> {
    loop {
        sync_sizes(terminal, app)?;

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
        if handle_pending(terminal, app)? {
            continue;
        }

        drain_events(app)?;

        if app.main_view.scroll + 50 >= app.main_view.highlighted_lines.len()
            && app.main_view.current_path.is_some()
        {
            app.ensure_full_content();
        }
        app.interactive.tabs.poll_output();
        poll_capture(app, &mut capture);
        app.file_tree
            .maybe_refresh(std::time::Duration::from_secs(2));
        app.reload_if_changed();
    }
}

fn sync_sizes(terminal: &Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    let size = terminal.size()?;
    let area = Rect::new(0, 0, size.width, size.height.saturating_sub(1));
    let c = LayoutConstraints::compute(area, app.layout_mode, &app.panel_sizes);
    app.interactive.sync_size(c.interactive);
    app.main_view.viewport_h = c.main.height.saturating_sub(2) as usize;
    Ok(())
}

fn handle_pending(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<bool> {
    if app.pending_editor.is_some() {
        app.run_pending_editor()?;
        terminal.clear()?;
        return Ok(true);
    }
    if app.pending_shell {
        app.run_pending_shell()?;
        terminal.clear()?;
        return Ok(true);
    }
    if app.pending_peek {
        app.pending_peek = false;
        peek_screen()?;
        terminal.clear()?;
        return Ok(true);
    }
    if app.pending_redraw {
        app.pending_redraw = false;
        terminal.clear()?;
        return Ok(true);
    }
    Ok(false)
}

fn drain_events(app: &mut App) -> Result<()> {
    if !event::poll(std::time::Duration::from_millis(50))? {
        return Ok(());
    }
    loop {
        match event::read()? {
            Event::Key(key) => app.handle_key(key)?,
            Event::Paste(text) => app.handle_paste(&text),
            _ => {}
        }
        if app.should_quit || !event::poll(std::time::Duration::from_millis(0))? {
            break;
        }
    }
    Ok(())
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
        Overlay::SaveFilePrompt(p) => {
            render_save_file_prompt(frame, p, Rect::new(x, area.y + 3, w, 3));
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

fn render_save_file_prompt(frame: &mut Frame, prompt: &overlay::SaveFilePrompt, area: Rect) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" Save to file (Enter to write, Esc to cancel) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let para = Paragraph::new(prompt.path.as_str()).block(block);
    frame.render_widget(para, area);
    let cx = (area.x + 1 + prompt.path.len() as u16).min(area.right().saturating_sub(2));
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
    let label = Style::default().fg(Color::LightCyan).bg(Color::Black);
    let value = Style::default().fg(Color::White).bg(Color::Black);

    let mut spans = status_left_spans(app, label, value);
    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let right = status_right_spans(app, label, value);
    let right_len: usize = right.iter().map(|s| s.content.len()).sum();

    let pad = area
        .width
        .saturating_sub(left_len as u16 + right_len as u16);
    spans.push(Span::styled(
        " ".repeat(pad as usize),
        Style::default().bg(Color::Black),
    ));
    spans.extend(right);
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn status_left_spans<'a>(app: &'a App, label: Style, value: Style) -> Vec<Span<'a>> {
    let focus_name = match app.focus {
        panel::FocusedPanel::Tree => "Tree",
        panel::FocusedPanel::Main => "Main",
        panel::FocusedPanel::Interactive => "Terminal",
    };
    let tree_mode = match app.left_mode {
        app::LeftPanelMode::FileTree => format!("Files:{}", app.file_tree.filter.label()),
        app::LeftPanelMode::CommitTree => "Commits".to_string(),
    };
    vec![
        Span::styled(" [", label),
        Span::styled(focus_name, value),
        Span::styled("  Left:", label),
        Span::styled(tree_mode, value),
        Span::styled("  Main:", label),
        Span::styled(app.main_view.mode.label(), value),
        Span::styled("  Tab:", label),
        Span::styled(app.interactive.tabs.active_title(), value),
    ]
}

fn status_right_spans(app: &App, label: Style, value: Style) -> Vec<Span<'static>> {
    let (tail_label, tail_value) = if app.focus == panel::FocusedPanel::Interactive {
        ("Esc²:".to_string(), "back")
    } else {
        (format!("{}:", app.config.display_key("quit")), "quit")
    };
    let mut spans = Vec::new();
    if let Some(prefix) = app.keymap.pending_label() {
        let pending_style = Style::default().fg(Color::Black).bg(Color::LightYellow);
        spans.push(Span::styled(format!(" {prefix}- "), pending_style));
    }
    spans.extend([
        Span::styled("C-S-↑/↓:", label),
        Span::styled("mode/tab  ", value),
        Span::styled("F3/4/5:", label),
        Span::styled("panel  ", value),
        Span::styled("F1:", label),
        Span::styled("help  ", value),
        Span::styled(tail_label, label),
        Span::styled(tail_value, value),
    ]);
    spans
}
