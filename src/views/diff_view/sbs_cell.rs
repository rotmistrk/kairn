//! SBS cell rendering — draws a single cell (left or right) in side-by-side mode.

use txv_core::cell::Style;
use txv_core::prelude::*;

use crate::views::editor::sbs_model::SbsLine;

/// Drawing context for one SBS cell.
pub(super) struct SbsCellCtx {
    pub(super) x: u16,
    pub(super) w: u16,
    pub(super) y: u16,
    pub(super) is_cursor: bool,
    pub(super) is_left: bool,
    pub(super) added: Style,
    pub(super) deleted: Style,
    pub(super) context: Style,
    pub(super) gap: Style,
    pub(super) cursor: Style,
}

pub(super) fn draw_sbs_cell(buf: &mut Buffer, line: &SbsLine, ctx: &SbsCellCtx) {
    match line {
        SbsLine::Content { text, changed, .. } => {
            let base_style = if *changed {
                if ctx.is_left {
                    ctx.deleted
                } else {
                    ctx.added
                }
            } else {
                ctx.context
            };
            let st = if ctx.is_cursor {
                ctx.cursor
            } else {
                base_style
            };
            buf.print_line(ctx.x, ctx.y, text, ctx.w, st);
        }
        SbsLine::Gap => {
            let st = if ctx.is_cursor {
                ctx.cursor
            } else {
                ctx.gap
            };
            buf.print_line(ctx.x, ctx.y, "", ctx.w, st);
        }
        SbsLine::Folded { count } => {
            let label = format!("--- {count} lines ---");
            buf.print_line(ctx.x, ctx.y, &label, ctx.w, ctx.context);
        }
    }
}
