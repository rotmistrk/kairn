//! MCP build/search handlers — Tier 5 operations.

use std::fs::read_to_string;
use std::path::Path;

use ignore::WalkBuilder;
use regex::RegexBuilder;
use txv_core::prelude::*;
use txv_core::run::Waker;

use crate::build::run_async;
use crate::build_detect::detect;
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

pub fn mcp_search_project(state: &AppState, pattern: &str, all_roots: bool) -> Result<serde_json::Value, String> {
    let re = RegexBuilder::new(pattern)
        .case_insensitive(false)
        .build()
        .map_err(|e| format!("Invalid regex: {e}"))?;
    let mut results = Vec::new();
    let dirs: Vec<&Path> = if all_roots && state.roots().len() > 1 {
        state.roots().all().iter().map(|r| r.path.as_path()).collect()
    } else {
        vec![state.root_dir.as_path()]
    };
    for dir in dirs {
        let walker = WalkBuilder::new(dir).hidden(true).git_ignore(true).build();
        for entry in walker.flatten() {
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }
            let path = entry.path();
            let Ok(content) = read_to_string(path) else {
                continue;
            };
            if let Some(val) = search_file_lines(&re, path, &content, dir, &mut results) {
                return Ok(val);
            }
        }
    }
    Ok(serde_json::json!({"matches": results, "truncated": false}))
}

fn search_file_lines(
    re: &regex::Regex,
    path: &std::path::Path,
    content: &str,
    root: &std::path::Path,
    results: &mut Vec<serde_json::Value>,
) -> Option<serde_json::Value> {
    for (i, line) in content.lines().enumerate() {
        if !re.is_match(line) {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(path);
        results.push(serde_json::json!({
            "file": rel.to_string_lossy(),
            "line": i + 1,
            "text": line,
        }));
        if results.len() >= 200 {
            return Some(serde_json::json!({"matches": results, "truncated": true}));
        }
    }
    None
}

pub fn mcp_run_build(state: &mut AppState, sink: &EventSink, command: &str) -> Result<serde_json::Value, String> {
    let cmd = if command.is_empty() {
        detect(&state.root_dir)
            .map(|bs| bs.build.to_string())
            .unwrap_or_else(|| "make".to_string())
    } else {
        command.to_string()
    };
    let waker = state.waker.clone().unwrap_or_else(Waker::noop);
    let task = run_async(&cmd, &state.root_dir, waker);
    state.build_pending = Some((cmd.clone(), task, state.root_dir.clone()));
    sink.push_command(
        txv_widgets::CM_STATUS_MESSAGE,
        Some(Box::new(Message::info("build", format!("Running: {cmd}")))),
    );
    Ok(serde_json::json!({"started": cmd}))
}
