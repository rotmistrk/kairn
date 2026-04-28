use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn csv_to_table(path: &str) -> (Vec<Line<'static>>, String) {
    let reader = csv::ReaderBuilder::new().flexible(true).from_path(path);
    let mut reader = match reader {
        Ok(r) => r,
        Err(e) => {
            let msg = format!("Cannot parse CSV: {e}");
            return (vec![Line::from(msg.clone())], msg);
        }
    };

    let headers: Vec<String> = reader
        .headers()
        .map(|h| h.iter().map(|s| s.to_string()).collect())
        .unwrap_or_default();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for record in reader.records().flatten() {
        rows.push(record.iter().map(|s| s.to_string()).collect());
    }

    let ncols = headers
        .len()
        .max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
    let widths = compute_col_widths(&headers, &rows, ncols);
    let numeric = detect_numeric_cols(&rows, ncols);
    render_table(&headers, &rows, &widths, &numeric)
}

fn compute_col_widths(
    headers: &[String],
    rows: &[Vec<String>],
    ncols: usize,
) -> Vec<usize> {
    let mut widths = vec![0usize; ncols];
    for (i, h) in headers.iter().enumerate() {
        widths[i] = widths[i].max(h.len());
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < ncols {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }
    widths
}

fn render_table(
    headers: &[String],
    rows: &[Vec<String>],
    widths: &[usize],
    numeric: &[bool],
) -> (Vec<Line<'static>>, String) {
    let mut styled = Vec::new();
    let mut raw = String::new();
    let hdr = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let sep = Style::default().fg(Color::DarkGray);
    let row_s = Style::default().fg(Color::White);

    let mut push = |text: String, style: Style| {
        raw.push_str(&text);
        raw.push('\n');
        styled.push(Line::from(Span::styled(text, style)));
    };

    push(fmt_row(headers, widths, numeric), hdr);
    let divider: String = widths
        .iter()
        .map(|w| "─".repeat(w + 2))
        .collect::<Vec<_>>()
        .join("┼");
    push(format!("─{divider}─"), sep);
    for row in rows {
        push(fmt_row(row, widths, numeric), row_s);
    }
    if rows.is_empty() && headers.is_empty() {
        styled.push(Line::from("(empty or not a CSV file)"));
    }
    (styled, raw)
}

fn detect_numeric_cols(rows: &[Vec<String>], ncols: usize) -> Vec<bool> {
    (0..ncols)
        .map(|i| {
            let non_empty: Vec<&str> = rows
                .iter()
                .filter_map(|r| r.get(i).map(|s| s.trim()))
                .filter(|s| !s.is_empty())
                .collect();
            if non_empty.is_empty() {
                return false;
            }
            non_empty.iter().all(|s| s.parse::<f64>().is_ok())
        })
        .collect()
}

fn fmt_row(cells: &[String], widths: &[usize], numeric: &[bool]) -> String {
    let parts: Vec<String> = widths
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let cell = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            if numeric.get(i).copied().unwrap_or(false) {
                format!(" {cell:>w$} ")
            } else {
                format!(" {cell:<w$} ")
            }
        })
        .collect();
    format!("│{}│", parts.join("│"))
}
