//! MCP build/search handlers — Tier 5 operations.

use txv_core::prelude::*;

use crate::handler::AppState;

pub fn mcp_get_build_errors(state: &AppState) -> Result<serde_json::Value, String> {
    let errors: Vec<serde_json::Value> = state
        .build_errors
        .iter()
        .map(|e| {
            serde_json::json!({
                "file": e.file,
                "line": e.line,
                "col": e.col,
                "message": e.message,
            })
        })
        .collect();
    Ok(serde_json::json!({"errors": errors}))
}

pub fn mcp_search_project(state: &AppState, pattern: &str) -> Result<serde_json::Value, String> {
    let re = regex::RegexBuilder::new(pattern)
        .case_insensitive(false)
        .build()
        .map_err(|e| format!("Invalid regex: {e}"))?;
    let mut results = Vec::new();
    let walker = ignore::WalkBuilder::new(&state.root_dir)
        .hidden(true)
        .git_ignore(true)
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let path = entry.path();
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for (i, line) in content.lines().enumerate() {
            if re.is_match(line) {
                let rel = path.strip_prefix(&state.root_dir).unwrap_or(path);
                results.push(serde_json::json!({
                    "file": rel.to_string_lossy(),
                    "line": i + 1,
                    "text": line,
                }));
                if results.len() >= 200 {
                    return Ok(serde_json::json!({"matches": results, "truncated": true}));
                }
            }
        }
    }
    Ok(serde_json::json!({"matches": results, "truncated": false}))
}

pub fn mcp_run_build(state: &mut AppState, sink: &EventSink, command: &str) -> Result<serde_json::Value, String> {
    let cmd = if command.is_empty() {
        crate::build_detect::detect(&state.root_dir)
            .map(|bs| bs.build.to_string())
            .unwrap_or_else(|| "make".to_string())
    } else {
        command.to_string()
    };
    let waker = state.waker.clone().unwrap_or_else(txv_core::run::Waker::noop);
    let task = crate::build::run_async(&cmd, &state.root_dir, waker);
    state.build_pending = Some((cmd.clone(), task, state.root_dir.clone()));
    sink.push_command(
        txv_widgets::CM_STATUS_MESSAGE,
        Some(Box::new(txv_core::message::Message::info(
            "build",
            format!("Running: {cmd}"),
        ))),
    );
    Ok(serde_json::json!({"started": cmd}))
}
