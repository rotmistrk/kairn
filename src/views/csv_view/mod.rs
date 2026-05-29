//! CsvView — tabular view for CSV/TSV files.

mod draw;
mod handle;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::input_line::InputLine;

use crate::csv_parse::{self, ColType, CsvData};

/// Tabular view for CSV/TSV files.
pub struct CsvView {
    pub(crate) group: GroupState,
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
    pub(crate) editing_row: Option<usize>,
    /// True when editing a filter, false when editing a cell.
    pub(crate) editing_filter: bool,
    pub(crate) dirty: bool,
    pub(crate) display_title: String,
    pub(crate) child_sink: EventSink,
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
            group: GroupState::default(),
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
            editing_row: None,
            editing_filter: false,
            dirty: false,
            display_title,
            child_sink: EventSink::new(),
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
        fs::write(&self.path, text).map_err(|e| e.to_string())
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.editing_row.is_some()
    }

    pub(crate) fn start_edit(&mut self) {
        if self.visible_rows.is_empty() {
            return;
        }
        self.cursor_row = self.cursor_row.min(self.visible_rows.len() - 1);
        let Some(&data_idx) = self.visible_rows.get(self.cursor_row) else {
            return;
        };
        if data_idx >= self.rows.len() {
            return;
        }
        let current = self.rows[data_idx].get(self.cursor_col).cloned().unwrap_or_default();
        self.editing_filter = false;
        self.insert_input_line(&current);
    }

    pub(crate) fn start_filter_edit(&mut self) {
        let text = self.filters.get(self.cursor_col).cloned().unwrap_or_default();
        self.editing_filter = true;
        self.insert_input_line(&text);
    }

    fn insert_input_line(&mut self, text: &str) {
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(text);
        input.select_all();
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        if let Some(child) = self.group.child_mut(0) {
            child.set_sink(sink);
            child.set_palette(pal);
        }
        self.editing_row = Some(self.cursor_row);
        self.group.mark_dirty();
    }

    pub(crate) fn cancel_edit(&mut self) {
        if self.group.child_count() > 0 {
            self.group.remove(0);
        }
        self.editing_row = None;
        self.editing_filter = false;
        self.group.mark_dirty();
    }

    fn edit_palette(&self) -> Arc<dyn Palette> {
        let base = palette();
        let cursor_style = base.style(StyleId::CursorFocused);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, cursor_style))
    }
}

impl View for CsvView {
    delegate_group_state!(group, override { title, draw, handle, set_bounds, cursor, select, unselect });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn set_bounds(&mut self, r: Rect) {
        if self.group.bounds() != r {
            self.cancel_edit();
        }
        self.group.set_bounds(r);
    }

    fn select(&mut self) {
        self.group.set_focused(true);
        self.group.mark_dirty();
    }

    fn unselect(&mut self) {
        self.group.set_focused(false);
        self.group.mark_dirty();
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        if self.is_editing() {
            return self.group.cursor();
        }
        None
    }

    fn draw(&mut self) {
        draw::draw_csv_view(self);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        handle::handle_csv_event(self, event)
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
    for w in &mut widths {
        *w = (*w).clamp(3, 40);
    }
    widths
}
