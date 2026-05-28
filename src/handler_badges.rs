//! Badge sync — dirty indicators and PTY activity badges.

use txv_core::program::CommandContext;

use crate::desktop::{close_tab_by_title, SlotId};
use crate::handler::{downcast_desktop, AppState};

/// Sync buffer dirty state to tab bar badges for center panel.
pub fn sync_dirty_badges(ctx: &mut CommandContext) {
    use crate::views::editor::EditorView;
    use crate::views::struct_view::StructuredView;

    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    for child_idx in 0..sp.child_count() {
        let Some(child) = sp.child_mut(child_idx) else {
            continue;
        };
        let Some(panel) = child
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<txv_widgets::tab_panel::TabPanel>())
        else {
            continue;
        };
        for i in 0..panel.tab_count() {
            let dirty = panel.view_at_mut(i).and_then(|v| v.as_any_mut()).is_some_and(|any| {
                if let Some(ev) = any.downcast_ref::<EditorView>() {
                    ev.editor.buf().is_dirty()
                } else if let Some(sv) = any.downcast_ref::<StructuredView>() {
                    sv.dirty
                } else {
                    false
                }
            });
            panel.set_dirty(i, dirty);
        }
    }
}

/// Auto-close exited terminal tabs in the tools panel.
pub fn auto_close_exited_terminals(ctx: &mut CommandContext, state: &mut AppState) {
    if !state.settings.terminal_auto_close {
        return;
    }
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    for slot in [SlotId::Tools] {
        let Some(panel) = desktop.panel(slot as usize) else {
            continue;
        };
        let titles: Vec<String> = (0..panel.tab_count())
            .filter_map(|i| {
                let t = desktop.panel(slot as usize)?.tab_title(i)?;
                if t.contains("[exited]") {
                    Some(t.to_string())
                } else {
                    None
                }
            })
            .collect();
        for title in titles {
            close_tab_by_title(desktop, slot, &title);
        }
    }
}

/// Animated spinner frames for busy terminals.
const SPINNER: &[char] = &['◐', '◑', '◒', '◓'];

/// Sync PTY activity badges on terminal tabs in the tools panel.
/// Running: animated spinner (green), Idle: ○ (yellow), Exited: ● (red).
pub fn sync_pty_badges(ctx: &mut CommandContext, state: &mut AppState) {
    let idle_secs = state.settings.terminal_idle_timeout;
    let now = std::time::Instant::now();
    let idle_dur = std::time::Duration::from_secs(idle_secs);
    let palette = &crate::app_palette::app_palette();
    let busy_style = palette.badge().busy();
    let idle_style = palette.badge().idle();
    let exited_style = palette.badge().exited();
    let frame = (state.mcp_tick / 4) as usize % SPINNER.len();

    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Tools as usize) else {
        return;
    };
    for i in 0..panel.tab_count() {
        let title = panel.tab_title(i).unwrap_or_default().to_string();
        let has_output = panel
            .view_at_mut(i)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_mut::<txv_widgets::pty_terminal::PtyTerminal>())
            .is_some_and(|pty| {
                let fresh = pty.has_fresh_output();
                pty.clear_output_flag();
                fresh
            });
        if has_output {
            state.pty_last_output.insert(i, now);
        }
        if title.contains("[exited]") {
            panel
                .bar_mut()
                .set_badge_styled(i, Some(" ●".to_string()), Some(exited_style));
        } else {
            let is_busy = state
                .pty_last_output
                .get(&i)
                .is_some_and(|&last| now.duration_since(last) <= idle_dur);
            if is_busy {
                let ch = SPINNER[frame];
                panel
                    .bar_mut()
                    .set_badge_styled(i, Some(format!(" {ch}")), Some(busy_style));
            } else if state.pty_last_output.contains_key(&i) {
                panel
                    .bar_mut()
                    .set_badge_styled(i, Some(" ○".to_string()), Some(idle_style));
            }
        }
    }
}
