//! VTE Perform trait implementation — dispatches terminal escape sequences.

use super::vte_handler::Performer;
use super::TCell;

impl vte::Perform for Performer<'_> {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => *self.cursor_x = 0,
            b'\x08' => {
                *self.cursor_x = self.cursor_x.saturating_sub(1);
            }
            b'\t' => {
                let next_tab = ((*self.cursor_x / 8) + 1) * 8;
                *self.cursor_x = next_tab.min(self.cols.saturating_sub(1));
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &vte::Params, intermediates: &[u8], _ignore: bool, action: char) {
        let ps: Vec<u16> = params.iter().map(|p| p[0]).collect();
        let p1 = ps.first().copied().unwrap_or(0);

        match (action, intermediates) {
            ('A', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_y = self.cursor_y.saturating_sub(n);
            }
            ('B', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_y = (*self.cursor_y + n).min(self.rows.saturating_sub(1));
            }
            ('C', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_x = (*self.cursor_x + n).min(self.cols.saturating_sub(1));
            }
            ('D', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_x = self.cursor_x.saturating_sub(n);
            }
            ('H' | 'f', []) => {
                let row = if p1 == 0 {
                    1
                } else {
                    p1
                };
                let col = ps.get(1).copied().unwrap_or(1).max(1);
                *self.cursor_y = (row - 1).min(self.rows.saturating_sub(1));
                *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1));
            }
            ('G', []) => {
                let col = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1));
            }
            ('J', []) => self.erase_display(p1),
            ('K', []) => self.erase_line(p1),
            ('L', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let bot = *self.scroll_bottom as usize;
                for _ in 0..n {
                    if y <= bot && bot < self.cells.len() {
                        self.cells.remove(bot);
                        self.cells.insert(y, vec![TCell::default(); self.cols as usize]);
                    }
                }
            }
            ('M', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let bot = *self.scroll_bottom as usize;
                for _ in 0..n {
                    if y <= bot && bot < self.cells.len() {
                        self.cells.remove(y);
                        self.cells.insert(bot, vec![TCell::default(); self.cols as usize]);
                    }
                }
            }
            ('S', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                for _ in 0..n {
                    self.scroll_up();
                }
            }
            ('T', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                for _ in 0..n {
                    self.scroll_down();
                }
            }
            ('m', []) => {
                if ps.is_empty() {
                    self.set_sgr(&[0]);
                } else {
                    self.set_sgr(&ps);
                }
            }
            ('r', []) => {
                let top = if p1 == 0 {
                    1
                } else {
                    p1
                };
                let bot = ps.get(1).copied().unwrap_or(self.rows).min(self.rows);
                *self.scroll_top = top.saturating_sub(1);
                *self.scroll_bottom = bot.saturating_sub(1);
                *self.cursor_x = 0;
                *self.cursor_y = 0;
            }
            ('h', [b'?']) => {
                if p1 == 25 {
                    *self.cursor_visible = true;
                }
            }
            ('l', [b'?']) => {
                if p1 == 25 {
                    *self.cursor_visible = false;
                }
            }
            ('s', []) => {
                *self.saved_cursor = (*self.cursor_x, *self.cursor_y);
            }
            ('u', []) => {
                *self.cursor_x = self.saved_cursor.0;
                *self.cursor_y = self.saved_cursor.1;
            }
            ('P', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let x = *self.cursor_x as usize;
                if y < self.rows as usize {
                    let row = &mut self.cells[y];
                    let end = (x + n).min(row.len());
                    row.drain(x..end);
                    row.resize(self.cols as usize, TCell::default());
                }
            }
            ('@', []) => {
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let x = *self.cursor_x as usize;
                if y < self.rows as usize {
                    let row = &mut self.cells[y];
                    for _ in 0..n {
                        if x < row.len() {
                            row.insert(x, TCell::default());
                        }
                    }
                    row.truncate(self.cols as usize);
                }
            }
            // DA1 — respond as VT100
            ('c', []) => {
                self.responses.push(b"\x1b[?1;2c".to_vec());
            }
            // DSR (CPR) — report cursor position
            ('n', []) if p1 == 6 => {
                let reply = format!("\x1b[{};{}R", *self.cursor_y + 1, *self.cursor_x + 1);
                self.responses.push(reply.into_bytes());
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (byte, intermediates) {
            (b'7', []) => {
                *self.saved_cursor = (*self.cursor_x, *self.cursor_y);
            }
            (b'8', []) => {
                *self.cursor_x = self.saved_cursor.0;
                *self.cursor_y = self.saved_cursor.1;
            }
            (b'D', []) => self.newline(),
            (b'M', []) => {
                if *self.cursor_y <= *self.scroll_top {
                    self.scroll_down();
                } else {
                    *self.cursor_y -= 1;
                }
            }
            // ESC k — tmux/screen title sequence; swallow until ST
            (b'k', []) => {
                *self.swallow_flag = true;
            }
            _ => {}
        }
    }
}
