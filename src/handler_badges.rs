//! Badge sync — dirty indicators and PTY activity badges.

use std::collections::{HashMap, HashSet};
use std::path::{PathBuf, MAIN_SEPARATOR};
use std::time::{Duration, Instant};

use txv_core::disambiguate::{disambiguate, Side};
use txv_core::program::CommandContext;
use txv_widgets::pty_terminal::PtyTerminal;
use txv_widgets::tab_panel::TabPanel;

use crate::app_palette::app_palette;
use crate::commands::CM_OPEN_FILES_CHANGED;
use crate::desktop::{close_tab_by_title, SlotId};
use crate::handler::{downcast_desktop, AppState};
use crate::views::diff_view::DiffView;
use crate::views::editor::EditorView;
use crate::views::struct_view::StructuredView;

/// Sync buffer dirty state to tab bar badges for center panel.
pub fn sync_dirty_badges(ctx: &mut CommandContext) {
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    for child_idx in 0..sp.child_count() {
        let Some(child) = sp.child_mut(child_idx) else {
            continue;
        };
        let Some(panel) = child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) else {
            continue;
        };
        for i in 0..panel.tab_count() {
            let dirty = panel.view_at_mut(i).and_then(|v| v.as_any_mut()).is_some_and(|any| {
                if let Some(ev) = any.downcast_ref::<EditorView>() {
                    ev.editor().buf().is_dirty()
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
    if !state.settings().terminal_auto_close() {
        return;
    }
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    for slot in [SlotId::Tools] {
        let Some(panel) = desktop.panel(slot as usize) else {
            continue;
        };
        let titles: Vec<String> = (0..panel.tab_count())
            .filter_map(|i| {
                let panel_ref = desktop.panel(slot as usize)?;
                let t = panel_ref.tab_title(i)?;
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
    let idle_secs = state.settings().terminal_idle_timeout();
    let now = Instant::now();
    let idle_dur = Duration::from_secs(idle_secs);
    let frame = (state.mcp().tick() / 16) as usize % SPINNER.len();

    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(SlotId::Tools as usize) else {
        return;
    };
    for i in 0..panel.tab_count() {
        update_pty_output_timestamp(panel, state, i, now);
        apply_pty_badge(panel, state, i, now, idle_dur, frame);
    }
}

fn update_pty_output_timestamp(
    panel: &mut txv_widgets::tab_panel::TabPanel,
    state: &mut AppState,
    i: usize,
    now: Instant,
) {
    let has_output = panel
        .view_at_mut(i)
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<PtyTerminal>())
        .is_some_and(|pty| {
            let fresh = pty.has_fresh_output();
            pty.clear_output_flag();
            fresh
        });
    if has_output {
        state.ui_mut().record_pty_output(i, now);
    }
}

fn apply_pty_badge(
    panel: &mut txv_widgets::tab_panel::TabPanel,
    state: &AppState,
    i: usize,
    now: Instant,
    idle_dur: Duration,
    frame: usize,
) {
    let palette = &app_palette();
    let title = panel.tab_title(i).unwrap_or_default().to_string();
    if title.contains("[exited]") {
        panel
            .bar_mut()
            .set_badge_styled(i, Some(" ●".to_string()), Some(palette.badge().exited()));
    } else {
        let is_busy = state
            .ui()
            .pty_last_output()
            .get(&i)
            .is_some_and(|&last| now.duration_since(last) <= idle_dur);
        if is_busy {
            let ch = SPINNER[frame];
            panel
                .bar_mut()
                .set_badge_styled(i, Some(format!(" {ch}")), Some(palette.badge().busy()));
        } else if state.ui().pty_last_output().contains_key(&i) {
            panel
                .bar_mut()
                .set_badge_styled(i, Some(" ○".to_string()), Some(palette.badge().idle()));
        }
    }
}

/// Recompute disambiguated tab titles when the set of open files changes.
pub fn sync_tab_titles(ctx: &mut CommandContext, state: &mut AppState) {
    let sink = ctx.sink().clone();
    if !state.ui().tab_titles_dirty() {
        return;
    }
    state.ui_mut().set_tab_titles_dirty(false);

    let root = state.root_dir().clone();
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    let mut open_set: HashSet<PathBuf> = HashSet::new();
    for child_idx in 0..sp.child_count() {
        let Some(child) = sp.child_mut(child_idx) else {
            continue;
        };
        let Some(panel) = child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) else {
            continue;
        };
        disambiguate_panel(panel, &root, &mut open_set);
    }
    sink.push_broadcast(CM_OPEN_FILES_CHANGED, Some(Box::new(open_set)));
}

/// Disambiguate tab titles within a single TabPanel.
fn disambiguate_panel(panel: &mut TabPanel, root: &std::path::Path, open_set: &mut HashSet<PathBuf>) {
    let rel_paths: Vec<String> = collect_rel_paths(panel, root, open_set);
    let unique: Vec<&str> = {
        let mut seen = HashSet::new();
        rel_paths
            .iter()
            .filter(|p| seen.insert(p.as_str()))
            .map(|p| p.as_str())
            .collect()
    };
    let labels = disambiguate(&unique, MAIN_SEPARATOR, Side::Right);
    let label_map: HashMap<&str, &str> = unique
        .iter()
        .zip(labels.iter())
        .map(|(&k, v)| (k, v.as_str()))
        .collect();
    for (i, rel) in rel_paths.iter().enumerate() {
        if let Some(&title) = label_map.get(rel.as_str()) {
            if !title.is_empty() {
                panel.set_title(i, title);
            }
        }
    }
}

/// Collect relative paths for each tab, inserting absolute paths into open_set.
fn collect_rel_paths(panel: &mut TabPanel, root: &std::path::Path, open_set: &mut HashSet<PathBuf>) -> Vec<String> {
    (0..panel.tab_count())
        .map(|i| {
            panel
                .view_at_mut(i)
                .and_then(|v| v.as_any_mut())
                .and_then(|any| {
                    any.downcast_ref::<EditorView>()
                        .map(|ev| ev.path().to_path_buf())
                        .or_else(|| any.downcast_ref::<StructuredView>().map(|sv| sv.path.clone()))
                })
                .map(|p| {
                    open_set.insert(p.clone());
                    p.strip_prefix(root).unwrap_or(&p).to_string_lossy().to_string()
                })
                .unwrap_or_default()
        })
        .collect()
}

/// Sync root color badges on editor tabs (only when multi-root is active).
pub fn sync_root_badges(ctx: &mut CommandContext, state: &AppState) {
    use txv_core::cell::Style;

    if state.roots().len() <= 1 {
        return;
    }
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let Some(sp) = desktop.split_panel_mut(SlotId::Center as usize) else {
        return;
    };
    for child_idx in 0..sp.child_count() {
        let Some(child) = sp.child_mut(child_idx) else {
            continue;
        };
        let Some(panel) = child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) else {
            continue;
        };
        for i in 0..panel.tab_count() {
            let color = panel
                .view_at_mut(i)
                .and_then(|v| v.as_any_mut())
                .and_then(|any| {
                    any.downcast_ref::<EditorView>()
                        .map(|ev| ev.path().to_path_buf())
                        .or_else(|| any.downcast_ref::<DiffView>().map(|dv| dv.path().clone()))
                })
                .map(|p| state.roots().root_for(&p).color());
            if let Some(c) = color {
                let style = Style::default().with_fg(c);
                panel.bar_mut().set_badge_styled(i, Some(" ●".to_string()), Some(style));
            }
        }
    }
}
