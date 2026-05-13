//! MCP snapshot collector — extracts state from the desktop for MCP tools.

use crate::layout_group::{LayoutGroup, SlotId};
use crate::mcp::snapshot::{CursorPos, McpSnapshot, SelectionRange, TabInfo, TerminalInfo};
use crate::views::editor::EditorView;

/// Collect current state from the desktop into a snapshot.
pub fn collect_snapshot(desktop: &mut LayoutGroup) -> McpSnapshot {
    let mut tabs = Vec::new();
    let focused_slot = desktop.focused_slot();
    let mut order = 0usize;

    for slot in [SlotId::Left, SlotId::Center, SlotId::Right, SlotId::Bottom] {
        let panel = desktop.panel_mut(slot);
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

    let slot_name = match focused_slot {
        SlotId::Left => "left",
        SlotId::Center => "center",
        SlotId::Right => "right",
        SlotId::Bottom => "bottom",
    };

    McpSnapshot {
        tabs,
        terminals: Vec::new(),
        focused_slot: slot_name.to_string(),
    }
}

/// Extract cursor, modified, and selection state from an editor view.
fn extract_editor_state(
    view: Option<&mut Box<dyn txv_core::view::View>>,
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
    let modified = editor.editor.buffer.is_dirty();
    let cursor = Some(CursorPos {
        line: editor.editor.cursor_line,
        col: editor.editor.cursor_col,
    });
    let selection = editor.editor.visual_range().map(|(start, end)| {
        let (sl, sc) = editor.editor.buffer.offset_to_line_col(start);
        let (el, ec) = editor.editor.buffer.offset_to_line_col(end);
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
pub fn collect_terminal_content(desktop: &mut LayoutGroup) -> Vec<TerminalInfo> {
    let mut terminals = Vec::new();
    let mut index = 0usize;
    for slot in [SlotId::Right, SlotId::Bottom] {
        let panel = desktop.panel_mut(slot);
        for i in 0..panel.tab_count() {
            let title = panel.tab_title(i).unwrap_or_default().to_string();
            let terminal_type = if title.starts_with("Kiro") {
                "kiro"
            } else {
                "shell"
            };
            if let Some(view) = panel.view_at_mut(i) {
                if let Some(any) = view.as_any_mut() {
                    if let Some(pty) = any.downcast_mut::<txv_widgets::PtyTerminal>() {
                        let content = pty.get_content(200).join("\n");
                        terminals.push(TerminalInfo {
                            name: title,
                            terminal_type: terminal_type.to_string(),
                            index,
                            content,
                        });
                        index += 1;
                    }
                }
            }
        }
    }
    terminals
}

fn classify_tab(slot: SlotId, title: &str) -> &'static str {
    match slot {
        SlotId::Left => "tree",
        SlotId::Right | SlotId::Bottom => {
            if title.starts_with("Kiro") {
                "kiro"
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
