//! CsvView — tabular view for CSV/TSV files.

mod draw;
mod format;
pub(crate) mod handle;
pub(crate) mod row_ops;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use txv_core::clipboard_ring::ClipboardHandle;
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
    /// Visual selection anchor (visible row index). None = not in visual mode.
    pub(crate) visual_anchor: Option<usize>,
    /// Yanked rows (internal buffer for copy/paste).
    pub(crate) yanked_rows: Vec<Vec<String>>,
    /// Clipboard handle (shared with app).
    pub(crate) clipboard: Option<ClipboardHandle>,
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
        let col_widths = format::compute_col_widths(&headers, &rows, ncols);
        let visible_rows: Vec<usize> = (0..rows.len()).collect();

        Self {
            group: GroupState::default(),
            path: path.to_path_buf(),
            display_title: file_title(path),
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
            filters: vec![String::new(); ncols],
            visible_rows,
            editing_row: None,
            editing_filter: false,
            dirty: false,
            child_sink: EventSink::new(),
            visual_anchor: None,
            yanked_rows: Vec::new(),
            clipboard: None,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn set_clipboard(&mut self, clipboard: ClipboardHandle) {
        self.clipboard = Some(clipboard);
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

    /// Returns (start, end) inclusive range of selected visible rows.
    pub(crate) fn visual_range(&self) -> Option<(usize, usize)> {
        let anchor = self.visual_anchor?;
        let a = anchor.min(self.cursor_row);
        let b = anchor.max(self.cursor_row);
        Some((a, b))
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
        self.group.set_focused_index(0);
        if let Some(child) = self.group.child_mut(0) {
            child.set_sink(sink);
            child.set_palette(pal);
            child.select();
        }
        self.editing_row = Some(self.cursor_row);
        self.layout_input_child();
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

    /// Position the InputLine child at the cursor cell.
    pub(crate) fn layout_input_child(&mut self) {
        if self.group.child_count() == 0 {
            return;
        }
        let header_offset: u16 = if self.headers.is_some() {
            1
        } else {
            0
        };
        let row = self.editing_row.unwrap_or(0);
        let screen_row = (row.saturating_sub(self.scroll_row)) as u16 + header_offset;
        let mut cx: u16 = 0;
        for (col_idx, &width) in self.col_widths.iter().enumerate() {
            if col_idx < self.scroll_col {
                continue;
            }
            if col_idx == self.cursor_col {
                break;
            }
            cx += width + 1;
        }
        let col_w = self.col_widths.get(self.cursor_col).copied().unwrap_or(10);
        self.group.set_child_bounds(0, Rect::new(cx, screen_row, col_w, 1));
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

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn set_bounds(&mut self, r: Rect) {
        if self.group.bounds() != r {
            self.cancel_edit();
        }
        self.group.set_bounds(r);
        self.layout_input_child();
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
        if self.is_editing() {
            let _result = self.group.dispatch(event);
            handle::drain_csv_commands(self);
            self.group.mark_dirty();
            return HandleResult::Consumed;
        }
        handle::handle_csv_event(self, event)
    }
}

fn file_title(path: &Path) -> String {
    path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "csv".into())
}
