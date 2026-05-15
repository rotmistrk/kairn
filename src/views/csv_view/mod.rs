//! CsvView — tabular view for CSV/TSV files.

mod draw;
mod handle;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;
use txv_widgets::inline_edit::InlineEditor;

use crate::csv_parse::{self, ColType, CsvData};

/// Tabular view for CSV/TSV files.
pub struct CsvView {
    pub(crate) state: ViewState,
    pub(crate) path: PathBuf,
    pub(crate) delimiter: char,
    pub(crate) headers: Option<Vec<String>>,
    pub(crate) rows: Vec<Vec<String>>,
    pub(crate) col_widths: Vec<u16>,
    pub(crate) col_types: Vec<ColType>,
    pub(crate) cursor_row: usize,
    pub(crate) cursor_col: usize,
    pub(crate) scroll_row: usize,
    pub(crate) scroll_col: usize,
    pub(crate) sort_col: Option<usize>,
    pub(crate) sort_asc: bool,
    pub(crate) filters: Vec<String>,
    pub(crate) visible_rows: Vec<usize>,
    pub(crate) editing: Option<InlineEditor>,
    pub(crate) dirty: bool,
    pub(crate) display_title: String,
}

impl CsvView {
    pub fn new(path: &Path, text: &str) -> Self {
        let CsvData {
            delimiter,
            headers,
            rows,
            col_types,
        } = csv_parse::parse_csv(text);
        let ncols = headers
            .as_ref()
            .map_or_else(|| rows.first().map_or(0, |r| r.len()), |h| h.len());
        let col_widths = compute_col_widths(&headers, &rows, ncols);
        let visible_rows: Vec<usize> = (0..rows.len()).collect();
        let filters = vec![String::new(); ncols];
        let display_title = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "csv".into());

        Self {
            state: ViewState::default(),
            path: path.to_path_buf(),
            delimiter,
            headers,
            rows,
            col_widths,
            col_types,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
            sort_col: None,
            sort_asc: true,
            filters,
            visible_rows,
            editing: None,
            dirty: false,
            display_title,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn ncols(&self) -> usize {
        self.col_widths.len()
    }

    pub(crate) fn refilter(&mut self) {
        self.visible_rows = (0..self.rows.len())
            .filter(|&i| {
                self.filters.iter().enumerate().all(|(col, f)| {
                    if f.is_empty() {
                        return true;
                    }
                    let val = self.rows[i].get(col).map(|s| s.as_str()).unwrap_or("");
                    val.to_lowercase().contains(&f.to_lowercase())
                })
            })
            .collect();
    }

    pub(crate) fn save(&self) -> Result<(), String> {
        let text = csv_parse::serialize(self.headers.as_deref(), &self.rows, self.delimiter);
        std::fs::write(&self.path, text).map_err(|e| e.to_string())
    }
}

impl View for CsvView {
    delegate_view_state!(state, override { title, needs_redraw });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn needs_redraw(&self) -> bool {
        true
    }

    fn draw(&self, surface: &mut Surface) {
        draw::draw_csv_view(self, surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        handle::handle_csv_event(self, event, queue)
    }
}

fn compute_col_widths(headers: &Option<Vec<String>>, rows: &[Vec<String>], ncols: usize) -> Vec<u16> {
    let mut widths = vec![0u16; ncols];
    if let Some(hdrs) = headers {
        for (i, h) in hdrs.iter().enumerate() {
            widths[i] = widths[i].max(h.len() as u16);
        }
    }
    for row in rows.iter().take(200) {
        for (i, cell) in row.iter().enumerate() {
            if i < ncols {
                widths[i] = widths[i].max(cell.len() as u16);
            }
        }
    }
    // Cap widths
    for w in &mut widths {
        *w = (*w).clamp(3, 40);
    }
    widths
}
