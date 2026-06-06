//! Cell formatting for CSV table view — alignment, decimal, scientific notation.

use crate::csv_parse::ColType;

pub(super) struct RowContext<'a> {
    pub(super) col_widths: &'a [u16],
    pub(super) col_types: &'a [ColType],
    pub(super) cells: &'a [String],
    pub(super) base: txv_core::cell::Style,
    pub(super) scroll_col: usize,
    pub(super) cursor_col: usize,
}

pub(super) fn format_cell(text: &str, width: usize, right_align: bool) -> String {
    let truncated = if text.len() > width {
        &text[..width]
    } else {
        text
    };
    if right_align {
        format!("{:>width$}", truncated, width = width)
    } else {
        format!("{:<width$}", truncated, width = width)
    }
}

/// Dot-aligned formatting for numeric columns.
pub(super) fn format_numeric_cell(text: &str, width: usize, col_type: &ColType) -> String {
    let ColType::Numeric {
        max_before_dot,
        max_after_dot,
        max_exp_width,
    } = col_type
    else {
        return format_cell(text, width, true);
    };
    let trimmed = text.trim();
    if trimmed.parse::<f64>().is_err() {
        return format_cell(text, width, true);
    }
    let aligned = align_numeric(trimmed, *max_before_dot, *max_after_dot, *max_exp_width);
    pad_to_width(&aligned, width)
}

fn align_numeric(trimmed: &str, max_before_dot: u16, max_after_dot: u16, max_exp_width: u16) -> String {
    let (mantissa, exp_part) = split_scientific_str(trimmed);
    let (before, after) = if let Some(dot) = mantissa.find('.') {
        (dot, mantissa.len() - dot - 1)
    } else {
        (mantissa.len(), 0)
    };

    let left_pad = (max_before_dot as usize).saturating_sub(before);
    let right_pad = if max_after_dot > 0 {
        if after == 0 {
            1 + max_after_dot as usize
        } else {
            (max_after_dot as usize).saturating_sub(after)
        }
    } else {
        0
    };
    let exp_pad = (max_exp_width as usize).saturating_sub(exp_part.len());

    let mut result = String::new();
    for _ in 0..left_pad {
        result.push(' ');
    }
    result.push_str(trimmed);
    for _ in 0..right_pad {
        result.push(' ');
    }
    for _ in 0..exp_pad {
        result.push(' ');
    }
    result
}

fn pad_to_width(s: &str, width: usize) -> String {
    if s.len() > width {
        s[..width].to_string()
    } else if s.len() < width {
        let pad = width - s.len();
        let mut out = String::with_capacity(width);
        for _ in 0..pad {
            out.push(' ');
        }
        out.push_str(s);
        out
    } else {
        s.to_string()
    }
}

/// Split numeric string at scientific notation exponent.
pub(super) fn split_scientific_str(s: &str) -> (&str, &str) {
    for (i, ch) in s.char_indices() {
        if i > 0 && (ch == 'e' || ch == 'E') {
            return (&s[..i], &s[i..]);
        }
    }
    (s, "")
}
