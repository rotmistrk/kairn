//! GitLogView drawing — tabular entry rendering and date formatting.

use std::time::{SystemTime, UNIX_EPOCH};

use txv_core::cell::{Attrs, Style};
use txv_core::palette::{palette, StyleId};

use crate::git_log::CommitEntry;

use super::git_log::GitLogView;

impl GitLogView {
    pub(super) fn draw_rows(&mut self, w: u16, h: u16) {
        let pal = palette();
        let normal = Style::default();
        let cursor_style = if self.state.is_focused() {
            pal.style(StyleId::CursorFocused)
        } else {
            pal.style(StyleId::CursorUnfocused)
        };
        let rows = h as usize;
        for row in 0..rows {
            let idx = self.scroll + row;
            let y = row as u16;
            if idx >= self.entries.len() {
                self.state.buffer_mut().hline(0, y, w, ' ', normal);
                continue;
            }
            let base = if idx == self.cursor {
                cursor_style
            } else {
                normal
            };
            self.draw_entry(y, w, &self.entries[idx].clone(), base);
        }
    }

    pub(super) fn draw_entry(&mut self, y: u16, w: u16, entry: &CommitEntry, base_style: Style) {
        let is_base = self.is_base_row(entry);
        let buf = self.state.buffer_mut();
        buf.hline(0, y, w, ' ', base_style);

        // Col 1: Badge (2 chars)
        let badge_ch = if is_base {
            '▶'
        } else {
            '●'
        };
        let badge_style = Style::new(entry.root_color, base_style.bg());
        buf.print(0, y, &format!("{badge_ch} "), badge_style);

        // Col 2: Date/time (10 chars + space)
        buf.print(2, y, &format_date_col(entry.time_secs), base_style);

        // Col 3: Hash (7 chars + space)
        let hash_style = if is_base {
            base_style.with_attrs(Attrs::default().underline())
        } else {
            base_style
        };
        buf.print(13, y, &entry.hash, hash_style);

        // Col 4: Author (12 chars + space)
        let author = truncate_str(&entry.author, 12);
        buf.print(21, y, &format!("{author:<12} "), base_style);

        // Col 5+6: Decorations + Summary
        let rest_start: u16 = 34;
        let rest_width = (w as usize).saturating_sub(rest_start as usize);
        let decor = if entry.decorations.is_empty() {
            String::new()
        } else {
            format!("({}) ", entry.decorations.join(", "))
        };
        let rest = truncate_str(&format!("{decor}{}", entry.summary), rest_width);
        buf.print(rest_start, y, &rest, base_style);
    }
}

fn format_date_col(epoch_secs: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff = (now - epoch_secs).max(0);
    if diff < 86400 {
        let secs = epoch_secs % 86400;
        let h = (secs / 3600) % 24;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{h:02}:{m:02}:{s:02}  ")
    } else {
        let days = epoch_secs / 86400;
        let (y, mo, d) = days_to_ymd(days);
        format!("{y:04}/{mo:02}/{d:02} ")
    }
}

fn days_to_ymd(days_since_epoch: i64) -> (i64, u32, u32) {
    // Civil days algorithm
    let z = days_since_epoch + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 {
        mp + 3
    } else {
        mp - 9
    };
    let y = if m <= 2 {
        y + 1
    } else {
        y
    };
    (y, m, d)
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}
