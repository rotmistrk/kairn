use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::config::Config;
use crate::diff::{BlameLine, DiffLine, DiffTag, LogEntry};

pub fn diff_lines_to_styled(lines: &[DiffLine]) -> Vec<Line<'static>> {
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

pub fn log_entries_to_styled(entries: &[LogEntry]) -> (Vec<Line<'static>>, String) {
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
        lines.push(Line::from("(no commits found)"));
        raw.push_str("(no commits found)\n");
    }

    (lines, raw)
}

fn log_entry_line(e: &LogEntry) -> Line<'static> {
    Line::from(vec![
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

pub fn welcome_lines(cfg: &Config) -> Vec<Line<'static>> {
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

fn welcome_banner() -> Vec<Line<'static>> {
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

fn welcome_keys(cfg: &Config) -> Vec<Line<'static>> {
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

pub fn blame_to_styled(lines: &[BlameLine]) -> (Vec<Line<'static>>, String) {
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

/// Copy text to system clipboard via OSC 52 escape sequence.
/// Works through ssh + tmux + iTerm2 chain.
pub fn osc52_copy(text: &str) {
    use std::io::Write;
    let encoded = base64_encode(text.as_bytes());
    let seq = format!("\x1b]52;c;{encoded}\x07");
    let _ = std::io::stdout().write_all(seq.as_bytes());
    let _ = std::io::stdout().flush();
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
