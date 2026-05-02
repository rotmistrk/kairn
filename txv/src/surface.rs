// Surface — bounded writable region of cells.

use crate::cell::{Cell, Span, Style};
use unicode_width::UnicodeWidthChar;

/// A bounded, writable rectangular region of cells.
/// Writes outside the bounds are silently clipped.
pub struct Surface<'a> {
    cells: &'a mut [Cell],
    stride: u16,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

impl<'a> Surface<'a> {
    /// Create a new surface over a cell slice.
    pub fn new(cells: &'a mut [Cell], stride: u16, x: u16, y: u16, w: u16, h: u16) -> Self {
        Self {
            cells,
            stride,
            x,
            y,
            w,
            h,
        }
    }

    /// Surface width.
    pub fn width(&self) -> u16 {
        self.w
    }

    /// Surface height.
    pub fn height(&self) -> u16 {
        self.h
    }

    /// Write a single character at (col, row) with style.
    /// Handles wide characters: writes char cell + continuation cell.
    pub fn put(&mut self, col: u16, row: u16, ch: char, style: Style) {
        if row >= self.h || col >= self.w {
            return;
        }
        let cw = ch.width().unwrap_or(1) as u8;
        if cw == 2 {
            // Wide char at last column: replace with space
            if col + 1 >= self.w {
                self.set_cell(col, row, ' ', style, 1);
                return;
            }
            self.set_cell(col, row, ch, style, 2);
            self.set_cell(col + 1, row, ' ', style, 0);
        } else {
            self.set_cell(col, row, ch, style, cw.max(1));
        }
    }

    /// Write a string starting at (col, row). Truncates at surface edge.
    /// Wide char at last column is replaced with a space.
    pub fn print(&mut self, col: u16, row: u16, text: &str, style: Style) {
        if row >= self.h {
            return;
        }
        let mut x = col;
        for ch in text.chars() {
            if x >= self.w {
                break;
            }
            let cw = ch.width().unwrap_or(0);
            if cw == 0 {
                continue;
            }
            if cw == 2 {
                if x + 1 >= self.w {
                    self.set_cell(x, row, ' ', style, 1);
                    break;
                }
                self.set_cell(x, row, ch, style, 2);
                self.set_cell(x + 1, row, ' ', style, 0);
                x += 2;
            } else {
                self.set_cell(x, row, ch, style, 1);
                x += 1;
            }
        }
    }

    /// Write styled spans starting at (col, row).
    pub fn print_spans(&mut self, col: u16, row: u16, spans: &[Span]) {
        if row >= self.h {
            return;
        }
        let mut x = col;
        for span in spans {
            for ch in span.text.chars() {
                if x >= self.w {
                    return;
                }
                let cw = ch.width().unwrap_or(0);
                if cw == 0 {
                    continue;
                }
                if cw == 2 {
                    if x + 1 >= self.w {
                        self.set_cell(x, row, ' ', span.style, 1);
                        return;
                    }
                    self.set_cell(x, row, ch, span.style, 2);
                    self.set_cell(x + 1, row, ' ', span.style, 0);
                    x += 2;
                } else {
                    self.set_cell(x, row, ch, span.style, 1);
                    x += 1;
                }
            }
        }
    }

    /// Fill the entire surface with a character and style.
    pub fn fill(&mut self, ch: char, style: Style) {
        for row in 0..self.h {
            for col in 0..self.w {
                self.set_cell(col, row, ch, style, 1);
            }
        }
    }

    /// Draw a horizontal line.
    pub fn hline(&mut self, col: u16, row: u16, len: u16, ch: char, style: Style) {
        if row >= self.h {
            return;
        }
        for i in 0..len {
            let x = col + i;
            if x >= self.w {
                break;
            }
            self.set_cell(x, row, ch, style, 1);
        }
    }

    /// Draw a vertical line.
    pub fn vline(&mut self, col: u16, row: u16, len: u16, ch: char, style: Style) {
        if col >= self.w {
            return;
        }
        for i in 0..len {
            let y = row + i;
            if y >= self.h {
                break;
            }
            self.set_cell(col, y, ch, style, 1);
        }
    }

    /// Create a sub-surface (further bounded region within this surface).
    /// Clips to the intersection of both bounds.
    pub fn sub(&mut self, col: u16, row: u16, w: u16, h: u16) -> Surface<'_> {
        let nx = self.x.saturating_add(col);
        let ny = self.y.saturating_add(row);
        // Clip to parent bounds
        let max_w = if col >= self.w {
            0
        } else {
            (self.w - col).min(w)
        };
        let max_h = if row >= self.h {
            0
        } else {
            (self.h - row).min(h)
        };
        Surface {
            cells: self.cells,
            stride: self.stride,
            x: nx,
            y: ny,
            w: max_w,
            h: max_h,
        }
    }

    fn set_cell(&mut self, col: u16, row: u16, ch: char, style: Style, width: u8) {
        let gx = (self.x + col) as usize;
        let gy = (self.y + row) as usize;
        let idx = gy * self.stride as usize + gx;
        if idx < self.cells.len() {
            self.cells[idx] = Cell { ch, style, width };
        }
    }
}

/// Read a cell from the underlying grid (for testing).
pub fn read_cell(cells: &[Cell], stride: u16, col: u16, row: u16) -> &Cell {
    let idx = row as usize * stride as usize + col as usize;
    &cells[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Style;

    fn make_grid(w: u16, h: u16) -> Vec<Cell> {
        vec![Cell::default(); w as usize * h as usize]
    }

    fn cell_at(grid: &[Cell], stride: u16, col: u16, row: u16) -> &Cell {
        read_cell(grid, stride, col, row)
    }

    #[test]
    fn put_writes_cell() {
        let mut grid = make_grid(10, 3);
        let mut s = Surface::new(&mut grid, 10, 0, 0, 10, 3);
        s.put(0, 0, 'H', Style::default());
        assert_eq!(cell_at(&grid, 10, 0, 0).ch, 'H');
    }

    #[test]
    fn put_wide_char() {
        let mut grid = make_grid(10, 1);
        let mut s = Surface::new(&mut grid, 10, 0, 0, 10, 1);
        s.put(2, 0, '漢', Style::default());
        assert_eq!(cell_at(&grid, 10, 2, 0).ch, '漢');
        assert_eq!(cell_at(&grid, 10, 2, 0).width, 2);
        assert_eq!(cell_at(&grid, 10, 3, 0).width, 0);
    }

    #[test]
    fn put_wide_at_last_col() {
        let mut grid = make_grid(5, 1);
        let mut s = Surface::new(&mut grid, 5, 0, 0, 5, 1);
        s.put(4, 0, '漢', Style::default());
        // Should be replaced with space
        assert_eq!(cell_at(&grid, 5, 4, 0).ch, ' ');
        assert_eq!(cell_at(&grid, 5, 4, 0).width, 1);
    }

    #[test]
    fn print_clips_at_edge() {
        let mut grid = make_grid(5, 1);
        let mut s = Surface::new(&mut grid, 5, 0, 0, 5, 1);
        s.print(3, 0, "Hello", Style::default());
        assert_eq!(cell_at(&grid, 5, 3, 0).ch, 'H');
        assert_eq!(cell_at(&grid, 5, 4, 0).ch, 'e');
    }

    #[test]
    fn print_wide_at_edge() {
        let mut grid = make_grid(5, 1);
        let mut s = Surface::new(&mut grid, 5, 0, 0, 5, 1);
        s.print(3, 0, "a漢", Style::default());
        assert_eq!(cell_at(&grid, 5, 3, 0).ch, 'a');
        // Wide char at col 4 can't fit (needs 2 cols), replaced with space
        assert_eq!(cell_at(&grid, 5, 4, 0).ch, ' ');
    }

    #[test]
    fn fill_all_cells() {
        let mut grid = make_grid(3, 2);
        let mut s = Surface::new(&mut grid, 3, 0, 0, 3, 2);
        s.fill('X', Style::default());
        for cell in &grid {
            assert_eq!(cell.ch, 'X');
        }
    }

    #[test]
    fn hline_draws() {
        let mut grid = make_grid(10, 3);
        let mut s = Surface::new(&mut grid, 10, 0, 0, 10, 3);
        s.hline(2, 1, 5, '─', Style::default());
        for i in 2..7 {
            assert_eq!(cell_at(&grid, 10, i, 1).ch, '─');
        }
        assert_eq!(cell_at(&grid, 10, 1, 1).ch, ' ');
        assert_eq!(cell_at(&grid, 10, 7, 1).ch, ' ');
    }

    #[test]
    fn vline_draws() {
        let mut grid = make_grid(5, 5);
        let mut s = Surface::new(&mut grid, 5, 0, 0, 5, 5);
        s.vline(2, 1, 3, '│', Style::default());
        for i in 1..4 {
            assert_eq!(cell_at(&grid, 5, 2, i).ch, '│');
        }
    }

    #[test]
    fn sub_surface_clips() {
        let mut grid = make_grid(10, 10);
        {
            let mut s = Surface::new(&mut grid, 10, 0, 0, 10, 10);
            let mut sub = s.sub(2, 2, 4, 4);
            assert_eq!(sub.width(), 4);
            assert_eq!(sub.height(), 4);
            sub.put(0, 0, 'A', Style::default());
            // Write outside sub bounds — should be clipped
            sub.put(5, 0, 'B', Style::default());
        }
        // 'A' should be at grid (2,2)
        assert_eq!(cell_at(&grid, 10, 2, 2).ch, 'A');
        // 'B' should not appear anywhere unexpected
        assert_eq!(cell_at(&grid, 10, 7, 2).ch, ' ');
    }

    #[test]
    fn sub_surface_nested() {
        let mut grid = make_grid(20, 20);
        {
            let mut s = Surface::new(&mut grid, 20, 0, 0, 20, 20);
            let mut sub1 = s.sub(5, 5, 10, 10);
            let mut sub2 = sub1.sub(2, 2, 4, 4);
            sub2.put(0, 0, 'Z', Style::default());
        }
        // Z should be at grid (5+2, 5+2) = (7, 7)
        assert_eq!(cell_at(&grid, 20, 7, 7).ch, 'Z');
    }

    #[test]
    fn print_spans_works() {
        let mut grid = make_grid(20, 1);
        let style_a = Style {
            fg: crate::cell::Color::Ansi(1),
            ..Style::default()
        };
        let style_b = Style {
            fg: crate::cell::Color::Ansi(2),
            ..Style::default()
        };
        let spans = [
            Span {
                text: "Hi",
                style: style_a,
            },
            Span {
                text: "Lo",
                style: style_b,
            },
        ];
        let mut s = Surface::new(&mut grid, 20, 0, 0, 20, 1);
        s.print_spans(0, 0, &spans);
        assert_eq!(cell_at(&grid, 20, 0, 0).ch, 'H');
        assert_eq!(
            cell_at(&grid, 20, 0, 0).style.fg,
            crate::cell::Color::Ansi(1)
        );
        assert_eq!(cell_at(&grid, 20, 2, 0).ch, 'L');
        assert_eq!(
            cell_at(&grid, 20, 2, 0).style.fg,
            crate::cell::Color::Ansi(2)
        );
    }

    #[test]
    fn put_out_of_bounds_no_panic() {
        let mut grid = make_grid(5, 5);
        let mut s = Surface::new(&mut grid, 5, 0, 0, 5, 5);
        s.put(10, 10, 'X', Style::default());
        // Should not panic, cell unchanged
    }

    #[test]
    fn sub_beyond_bounds_empty() {
        let mut grid = make_grid(5, 5);
        let mut s = Surface::new(&mut grid, 5, 0, 0, 5, 5);
        let sub = s.sub(10, 10, 5, 5);
        assert_eq!(sub.width(), 0);
        assert_eq!(sub.height(), 0);
    }
}
