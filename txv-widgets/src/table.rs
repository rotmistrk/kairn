//! Table widget — column headers + rows with alignment and selection.

use crossterm::event::{KeyCode, KeyEvent};
use txv::cell::Style;
use txv::surface::Surface;
use txv::text::display_width;

use crate::scroll_view::ScrollView;
use crate::widget::{EventResult, Widget, WidgetAction};

/// Column alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// Left-aligned (default).
    Left,
    /// Right-aligned.
    Right,
    /// Centered.
    Center,
}

/// Column definition.
#[derive(Clone, Debug)]
pub struct Column {
    /// Header text.
    pub title: String,
    /// Fixed width (0 = auto-fill remaining space).
    pub width: u16,
    /// Text alignment.
    pub align: Align,
}

/// Table widget with headers, rows, and single-row selection.
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    selected: usize,
    scroll: ScrollView,
    /// Style for the header row.
    pub header_style: Style,
    /// Style for normal rows.
    pub row_style: Style,
    /// Style for the selected row.
    pub selected_style: Style,
}

impl Table {
    /// Create a new table with the given columns.
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            selected: 0,
            scroll: ScrollView::new(),
            header_style: Style {
                attrs: txv::cell::Attrs {
                    bold: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            row_style: Style::default(),
            selected_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
        }
    }

    /// Set the row data. Each row is a vec of cell strings.
    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        self.scroll.set_content_size(rows.len(), 0);
        self.rows = rows;
        self.selected = 0;
        self.scroll.scroll_to_top();
    }

    /// Number of data rows.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Currently selected row index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected row.
    pub fn set_selected(&mut self, index: usize) {
        if !self.rows.is_empty() {
            self.selected = index.min(self.rows.len().saturating_sub(1));
        }
    }

    /// Get a row by index.
    pub fn row(&self, index: usize) -> Option<&[String]> {
        self.rows.get(index).map(|r| r.as_slice())
    }

    fn compute_widths(&self, total: u16) -> Vec<u16> {
        let fixed_total: u16 = self.columns.iter().map(|c| c.width).sum();
        let fill_count = self.columns.iter().filter(|c| c.width == 0).count() as u16;
        let remaining = total.saturating_sub(fixed_total);
        let per_fill = if fill_count > 0 {
            remaining / fill_count
        } else {
            0
        };
        let mut extra = if fill_count > 0 {
            remaining % fill_count
        } else {
            0
        };

        self.columns
            .iter()
            .map(|c| {
                if c.width > 0 {
                    c.width
                } else {
                    let w = per_fill + if extra > 0 { 1 } else { 0 };
                    extra = extra.saturating_sub(1);
                    w
                }
            })
            .collect()
    }

    fn render_row_cells(
        surface: &mut Surface<'_>,
        cells: &[String],
        columns: &[Column],
        widths: &[u16],
        style: Style,
        row: u16,
    ) {
        let mut col: u16 = 0;
        for (i, width) in widths.iter().enumerate() {
            let w = *width;
            if w == 0 || col >= surface.width() {
                break;
            }
            let text = cells.get(i).map(|s| s.as_str()).unwrap_or("");
            let align = columns.get(i).map(|c| c.align).unwrap_or(Align::Left);
            let tw = display_width(text) as u16;
            let avail = w.min(surface.width().saturating_sub(col));

            // Fill cell background
            surface.hline(col, row, avail, ' ', style);

            let x = match align {
                Align::Left => col,
                Align::Right => col + avail.saturating_sub(tw),
                Align::Center => col + avail.saturating_sub(tw) / 2,
            };
            surface.print(x, row, text, style);
            col += w;
        }
        // Fill remaining width
        if col < surface.width() {
            surface.hline(col, row, surface.width() - col, ' ', style);
        }
    }
}

impl Widget for Table {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let w = surface.width();
        let h = surface.height();
        if h == 0 {
            return;
        }
        let widths = self.compute_widths(w);

        // Header row
        let headers: Vec<String> = self.columns.iter().map(|c| c.title.clone()).collect();
        Self::render_row_cells(
            surface,
            &headers,
            &self.columns,
            &widths,
            self.header_style,
            0,
        );

        // Data rows
        let data_h = h.saturating_sub(1);
        let range = self.scroll.visible_range(data_h);
        for (row_idx, data_idx) in range.enumerate() {
            let style = if data_idx == self.selected {
                self.selected_style
            } else {
                self.row_style
            };
            let cells = self.rows.get(data_idx).map(|r| r.as_slice()).unwrap_or(&[]);
            Self::render_row_cells(
                surface,
                cells,
                &self.columns,
                &widths,
                style,
                (row_idx + 1) as u16,
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if self.rows.is_empty() {
            return match key.code {
                KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
                _ => EventResult::Ignored,
            };
        }
        match key.code {
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Down => {
                let max = self.rows.len().saturating_sub(1);
                self.selected = (self.selected + 1).min(max);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(10);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                let max = self.rows.len().saturating_sub(1);
                self.selected = (self.selected + 10).min(max);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.selected = 0;
                self.scroll.ensure_visible(0, 0);
                EventResult::Consumed
            }
            KeyCode::End => {
                self.selected = self.rows.len().saturating_sub(1);
                self.scroll.ensure_visible(self.selected, 0);
                EventResult::Consumed
            }
            KeyCode::Enter => {
                EventResult::Action(WidgetAction::Selected(self.selected.to_string()))
            }
            KeyCode::Esc => EventResult::Action(WidgetAction::Cancelled),
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn sample_columns() -> Vec<Column> {
        vec![
            Column {
                title: "Name".into(),
                width: 10,
                align: Align::Left,
            },
            Column {
                title: "Size".into(),
                width: 8,
                align: Align::Right,
            },
            Column {
                title: "Type".into(),
                width: 0,
                align: Align::Left,
            },
        ]
    }

    fn sample_rows() -> Vec<Vec<String>> {
        vec![
            vec!["foo.rs".into(), "1234".into(), "Rust".into()],
            vec!["bar.go".into(), "5678".into(), "Go".into()],
            vec!["baz.py".into(), "90".into(), "Python".into()],
        ]
    }

    fn render_table(table: &Table, w: u16, h: u16) -> String {
        let mut screen = Screen::with_color_mode(w, h, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            table.render(&mut s, true);
        }
        screen.to_text()
    }

    #[test]
    fn new_empty() {
        let t = Table::new(sample_columns());
        assert_eq!(t.row_count(), 0);
        assert_eq!(t.selected(), 0);
    }

    #[test]
    fn set_rows_and_count() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        assert_eq!(t.row_count(), 3);
    }

    #[test]
    fn render_shows_headers() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        let text = render_table(&t, 40, 5);
        assert!(text.contains("Name"));
        assert!(text.contains("Size"));
        assert!(text.contains("Type"));
    }

    #[test]
    fn render_shows_data() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        let text = render_table(&t, 40, 5);
        assert!(text.contains("foo.rs"));
        assert!(text.contains("bar.go"));
    }

    #[test]
    fn right_align() {
        let mut t = Table::new(vec![Column {
            title: "Num".into(),
            width: 10,
            align: Align::Right,
        }]);
        t.set_rows(vec![vec!["42".into()]]);
        let mut screen = Screen::with_color_mode(10, 2, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            t.render(&mut s, true);
        }
        // "42" right-aligned in 10 cols: should be at col 8
        assert_eq!(screen.cell(8, 1).ch, '4');
        assert_eq!(screen.cell(9, 1).ch, '2');
    }

    #[test]
    fn center_align() {
        let mut t = Table::new(vec![Column {
            title: "X".into(),
            width: 10,
            align: Align::Center,
        }]);
        t.set_rows(vec![vec!["ab".into()]]);
        let mut screen = Screen::with_color_mode(10, 2, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            t.render(&mut s, true);
        }
        // "ab" centered in 10: (10-2)/2 = 4
        assert_eq!(screen.cell(4, 1).ch, 'a');
        assert_eq!(screen.cell(5, 1).ch, 'b');
    }

    #[test]
    fn navigation_up_down() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        assert_eq!(t.selected(), 0);
        t.handle_key(key(KeyCode::Down));
        assert_eq!(t.selected(), 1);
        t.handle_key(key(KeyCode::Down));
        assert_eq!(t.selected(), 2);
        t.handle_key(key(KeyCode::Down)); // clamped
        assert_eq!(t.selected(), 2);
        t.handle_key(key(KeyCode::Up));
        assert_eq!(t.selected(), 1);
    }

    #[test]
    fn home_end() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        t.handle_key(key(KeyCode::End));
        assert_eq!(t.selected(), 2);
        t.handle_key(key(KeyCode::Home));
        assert_eq!(t.selected(), 0);
    }

    #[test]
    fn enter_selects() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        t.handle_key(key(KeyCode::Down));
        let result = t.handle_key(key(KeyCode::Enter));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Selected(s)) if s == "1"
        ));
    }

    #[test]
    fn esc_cancels() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        let result = t.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn empty_table_esc() {
        let mut t = Table::new(sample_columns());
        let result = t.handle_key(key(KeyCode::Esc));
        assert!(matches!(
            result,
            EventResult::Action(WidgetAction::Cancelled)
        ));
    }

    #[test]
    fn empty_table_down_ignored() {
        let mut t = Table::new(sample_columns());
        let result = t.handle_key(key(KeyCode::Down));
        assert!(matches!(result, EventResult::Ignored));
    }

    #[test]
    fn fill_column_takes_remaining() {
        let cols = vec![
            Column {
                title: "A".into(),
                width: 10,
                align: Align::Left,
            },
            Column {
                title: "B".into(),
                width: 0,
                align: Align::Left,
            },
        ];
        let t = Table::new(cols);
        let widths = t.compute_widths(30);
        assert_eq!(widths[0], 10);
        assert_eq!(widths[1], 20);
    }

    #[test]
    fn set_selected_clamps() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        t.set_selected(100);
        assert_eq!(t.selected(), 2);
    }

    #[test]
    fn get_row() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        assert_eq!(t.row(0).map(|r| &r[0]), Some(&"foo.rs".to_string()));
        assert!(t.row(10).is_none());
    }

    #[test]
    fn selected_row_has_reverse() {
        let mut t = Table::new(sample_columns());
        t.set_rows(sample_rows());
        let mut screen = Screen::with_color_mode(30, 5, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            t.render(&mut s, true);
        }
        // Row 0 header (bold), row 1 = first data row (selected, reverse)
        assert!(screen.cell(0, 1).style.attrs.reverse);
        assert!(!screen.cell(0, 2).style.attrs.reverse);
    }
}
