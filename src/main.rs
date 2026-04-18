// Deny unwrap/expect in non-test code — forces proper error handling.
#![deny(clippy::unwrap_used, clippy::expect_used)]
// Allow dead code during scaffold phase — remove once features are wired up.
#![allow(dead_code)]

mod app;
mod buffer;
mod config;
mod diff;
mod editor;
mod highlight;
mod input;
mod keymap;
mod layout;
mod overlay;
mod panel;
mod search;
mod session;
mod tab;
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| ".".into())
        .to_string_lossy()
        .to_string();
    let mut app = App::new(cwd);

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
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
            return Ok(());
        }

        if app.pending_editor.is_some() {
            app.run_pending_editor()?;
            terminal.clear()?;
            continue;
        }

        // Poll for keyboard input with short timeout
        // so we can also read PTY output between frames
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key)?;
            }
        }

        // Read any pending output from all tab backends
        app.interactive.tabs.poll_output();
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
    frame.set_cursor_position((input_area.x + 1 + search.cursor as u16, input_area.y + 1));
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
    frame.set_cursor_position((area.x + 1 + prompt.cursor as u16, area.y + 1));
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
