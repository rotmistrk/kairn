//! MCP snapshot collector — extracts state from the desktop for MCP tools.

use std::collections::HashMap;

use txv_widgets::tiled_workspace::types::SplitDir;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::desktop::{slot_from, SlotId};
use crate::mcp::snapshot::{CursorPos, McpSnapshot, SelectionRange, TabInfo, TerminalInfo};
use crate::message_ring::MsgLevel;
use crate::views::editor::EditorView;
use crate::views::problems::ProblemsView;
use crate::views::results::ResultsView;

/// Collect current state from the desktop into a snapshot.
pub fn collect_snapshot(desktop: &mut TiledWorkspace) -> McpSnapshot {
    let tabs = collect_tabs(desktop);
    let focused_slot = slot_from(desktop.focused_panel());

    let slot_name = match focused_slot {
        SlotId::Left => "left",
        SlotId::Center => "center",
        SlotId::Tools => "right",
    };

    let (split_direction, split_linked) = collect_split_state(desktop);

    McpSnapshot {
        tabs,
        terminals: Vec::new(),
        focused_slot: slot_name.to_string(),
        messages: Vec::new(),
        tab_contents: collect_center_contents(desktop),
        split_direction,
        split_linked,
    }
}

fn collect_tabs(desktop: &mut TiledWorkspace) -> Vec<TabInfo> {
    let mut tabs = Vec::new();
    let focused_slot = slot_from(desktop.focused_panel());
    let mut order = 0usize;

    for slot in [SlotId::Left, SlotId::Center, SlotId::Tools] {
        let Some(panel) = desktop.panel_mut(slot as usize) else {
            continue;
        };
        let active_idx = panel.active_index();
        for i in 0..panel.tab_count() {
            let title = panel.tab_title(i).unwrap_or_default().to_string();
            let tab_type = classify_tab(slot, &title);
            let path = if tab_type == "editor" {
                Some(title.clone())
            } else {
                None
            };
            let is_focused = slot == focused_slot && i == active_idx;

            let (modified, cursor, selection) = if tab_type == "editor" {
                extract_editor_state(panel.view_at_mut(i))
            } else {
                (false, None, None)
            };

            tabs.push(TabInfo {
                name: title,
                tab_type: tab_type.to_string(),
                path,
                focused: is_focused,
                active: i == active_idx,
                modified,
                cursor,
                selection,
                order,
            });
            order += 1;
        }
    }
    tabs
}

fn collect_split_state(desktop: &mut TiledWorkspace) -> (String, bool) {
    let is_split = desktop
        .split_panel(SlotId::Center as usize)
        .map(|sp| sp.child_count() > 1)
        .unwrap_or(false);
    if is_split {
        let dir = desktop
            .split_panel(SlotId::Center as usize)
            .map(|sp| match sp.direction() {
                SplitDir::Horizontal => "horizontal",
                SplitDir::Vertical => "vertical",
            })
            .unwrap_or("none");
        (dir.to_string(), false)
    } else {
        ("none".to_string(), false)
    }
}

/// Extract cursor, modified, and selection state from an editor view.
fn extract_editor_state(
    view: Option<&mut (dyn txv_core::view::View + '_)>,
) -> (bool, Option<CursorPos>, Option<SelectionRange>) {
    let Some(v) = view else {
        return (false, None, None);
    };
    let Some(any) = v.as_any_mut() else {
        return (false, None, None);
    };
    let Some(editor) = any.downcast_ref::<EditorView>() else {
        return (false, None, None);
    };
    let modified = editor.editor().buf().is_dirty();
    let cursor = Some(CursorPos {
        line: editor.editor().cursor_line(),
        col: editor.editor().cursor_col(),
    });
    let selection = editor.editor().visual_range().map(|(start, end)| {
        let (sl, sc) = editor.editor().buf().offset_to_line_col(start);
        let (el, ec) = editor.editor().buf().offset_to_line_col(end);
        SelectionRange {
            start_line: sl,
            start_col: sc,
            end_line: el,
            end_col: ec,
        }
    });
    (modified, cursor, selection)
}

/// Collect terminal content (requires mutable access for PtyTerminal).
pub fn collect_terminal_content(desktop: &mut TiledWorkspace) -> Vec<TerminalInfo> {
    let mut terminals = Vec::new();
    let mut index = 0usize;
    let Some(panel) = desktop.panel_mut(SlotId::Tools as usize) else {
        return terminals;
    };
    for i in 0..panel.tab_count() {
        let title = panel.tab_title(i).unwrap_or_default().to_string();
        let terminal_type = if title.starts_with("Kiro") {
            "kiro"
        } else {
            "shell"
        };
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        let Some(pty) = any.downcast_mut::<txv_widgets::PtyTerminal>() else {
            continue;
        };
        let content = pty.get_content(200).join("\n");
        terminals.push(TerminalInfo {
            name: title,
            terminal_type: terminal_type.to_string(),
            index,
            content,
        });
        index += 1;
    }
    terminals
}

fn classify_tab(slot: SlotId, title: &str) -> &'static str {
    match slot {
        SlotId::Left => "tree",
        SlotId::Tools => {
            if title.starts_with("Kiro") {
                "kiro"
            } else if title == "Problems" || title == "Messages" {
                "view"
            } else {
                "shell"
            }
        }
        SlotId::Center => {
            if title == "Help" || title == "Welcome" || title.starts_with("grep:") || title.starts_with("Ref") {
                "view"
            } else {
                "editor"
            }
        }
    }
}

/// Collect content from all panel tabs for the snapshot.
fn collect_center_contents(desktop: &mut TiledWorkspace) -> HashMap<String, String> {
    let mut contents = HashMap::new();
    for slot in [SlotId::Center, SlotId::Tools, SlotId::Left] {
        let Some(panel) = desktop.panel_mut(slot as usize) else {
            continue;
        };
        for i in 0..panel.tab_count() {
            let title = panel.tab_title(i).unwrap_or_default().to_string();
            if contents.contains_key(&title) {
                continue;
            }
            let Some(view) = panel.view_at_mut(i) else {
                continue;
            };
            let Some(any) = view.as_any_mut() else {
                continue;
            };
            if let Some(editor) = any.downcast_ref::<EditorView>() {
                contents.insert(title, editor.editor().buf().content());
            } else if let Some(results) = any.downcast_ref::<ResultsView>() {
                let lines: Vec<String> = results
                    .entries()
                    .iter()
                    .map(|e| format!("{}:{}:{}: {}", e.path.display(), e.line + 1, e.col + 1, e.text))
                    .collect();
                contents.insert(title, lines.join("\n"));
            } else if let Some(problems) = any.downcast_ref::<ProblemsView>() {
                contents.insert(title, problems.format_for_mcp());
            }
        }
    }
    contents
}

/// Format messages for MCP snapshot.
pub fn collect_messages(ring: &std::sync::Arc<std::sync::Mutex<crate::message_ring::MessageRing>>) -> Vec<String> {
    let Ok(guard) = ring.lock() else {
        return Vec::new();
    };
    guard
        .entries()
        .iter()
        .map(|m| {
            let level = match m.level() {
                MsgLevel::Error => "ERR",
                MsgLevel::Warn => "WARN",
                MsgLevel::Info => "INFO",
                MsgLevel::Debug => "DBG",
            };
            if m.count() > 1 {
                format!("[{}] [{}] {} (x{})", level, m.origin(), m.text(), m.count())
            } else {
                format!("[{}] [{}] {}", level, m.origin(), m.text())
            }
        })
        .collect()
}
