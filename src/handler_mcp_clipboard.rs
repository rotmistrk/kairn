//! MCP clipboard handlers.

use crate::handler::AppState;

pub(crate) fn mcp_clipboard_copy(state: &mut AppState, text: &str, source: &str) -> Result<serde_json::Value, String> {
    if let Ok(mut ring) = state.clipboard.lock() {
        ring.push(text, source);
    }
    Ok(serde_json::json!({"ok": true}))
}

pub(crate) fn mcp_clipboard_paste(state: &mut AppState) -> Result<serde_json::Value, String> {
    let text = state
        .clipboard
        .lock()
        .ok()
        .and_then(|mut r| r.paste())
        .unwrap_or_default();
    Ok(serde_json::json!({"text": text}))
}

pub(crate) fn mcp_clipboard_list(state: &mut AppState) -> Result<serde_json::Value, String> {
    let entries: Vec<serde_json::Value> = state
        .clipboard
        .lock()
        .map(|r| {
            r.entries()
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    serde_json::json!({
                        "index": i,
                        "first_line": e.text().lines().next().unwrap_or(""),
                        "line_count": e.line_count(),
                        "source": e.source(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(serde_json::json!({"entries": entries}))
}
