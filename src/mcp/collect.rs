//! MCP snapshot collector — extracts state from the desktop for MCP tools.

use crate::layout_group::{LayoutGroup, SlotId};
use crate::mcp::snapshot::{McpSnapshot, TabInfo, TerminalInfo};

/// Collect current state from the desktop into a snapshot.
pub fn collect_snapshot(desktop: &LayoutGroup) -> McpSnapshot {
    let mut tabs = Vec::new();
    let mut terminals = Vec::new();

    for slot in [SlotId::Left, SlotId::Center, SlotId::Right, SlotId::Bottom] {
        let panel = desktop.panel(slot);
        for i in 0..panel.tab_count() {
            let title = panel.tab_title(i).unwrap_or_default().to_string();
            let tab_type = classify_tab(slot, &title);
            let path = if tab_type == "editor" { Some(title.clone()) } else { None };
            tabs.push(TabInfo { name: title.clone(), tab_type: tab_type.to_string(), path });

            if tab_type == "shell" || tab_type == "kiro" {
                terminals.push(TerminalInfo {
                    name: title,
                    terminal_type: tab_type.to_string(),
                    content: String::new(), // populated below via mut access
                });
            }
        }
    }

    McpSnapshot { tabs, terminals }
}

/// Collect terminal content (requires mutable access for PtyTerminal).
pub fn collect_terminal_content(desktop: &mut LayoutGroup) -> Vec<TerminalInfo> {
    let mut terminals = Vec::new();
    for slot in [SlotId::Right, SlotId::Bottom] {
        let panel = desktop.panel_mut(slot);
        for i in 0..panel.tab_count() {
            let title = panel.tab_title(i).unwrap_or_default().to_string();
            let tab_type = if title.starts_with("Kiro") { "kiro" } else { "shell" };
            if let Some(view) = panel.view_at_mut(i) {
                if let Some(any) = view.as_any_mut() {
                    if let Some(pty) = any.downcast_mut::<txv_widgets::PtyTerminal>() {
                        let content = pty.get_content(200).join("\n");
                        terminals.push(TerminalInfo {
                            name: title,
                            terminal_type: tab_type.to_string(),
                            content,
                        });
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
            if title.starts_with("Kiro") { "kiro" } else { "shell" }
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
