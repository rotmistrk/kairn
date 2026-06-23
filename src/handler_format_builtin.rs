//! Built-in formatter: pretty-print JSON/JSONC/YAML without LSP.

use std::path::Path;

use serde::Serialize;
use serde_json::ser::PrettyFormatter;
use serde_json::{Serializer, Value};
use txv_core::message::Message;
use txv_core::program::CommandContext;
use txv_core::view::EventSink;

use crate::commands::CM_LSP_FORMAT_RESULT;
use crate::handler::{downcast_desktop, AppState};
use crate::views::editor::EditorView;

pub(crate) fn cmd_format_builtin(ctx: &mut CommandContext, _state: &mut AppState, arg: &str) {
    let opts = parse_format_opts(arg);

    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    let slot = desktop.focused_panel();
    let Some(view) = desktop.panel_mut(slot).and_then(|p| p.active_view_mut()) else {
        return;
    };
    let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) else {
        return;
    };

    let content = editor.content();
    let ext = extension(editor.path());

    let formatted = match ext {
        "json" | "jsonc" => format_json(&content, &opts),
        _ => Err(format!("No built-in formatter for .{ext} files")),
    };

    let sink = ctx.sink().clone();
    match formatted {
        Ok(text) => push_format_edit(&sink, &content, &text),
        Err(e) => {
            sink.push_command(
                txv_widgets::CM_STATUS_MESSAGE,
                Some(Box::new(Message::error("fmt!", e))),
            );
        }
    }
}

fn push_format_edit(sink: &EventSink, content: &str, text: &str) {
    let line_count = content.lines().count();
    let last_line = line_count.saturating_sub(1);
    let last_col = content.lines().last().map_or(0, |l| l.len());
    let edit = serde_json::json!([{
        "range": {
            "start": {"line": 0, "character": 0},
            "end": {"line": last_line, "character": last_col}
        },
        "newText": text
    }]);
    sink.push_command(CM_LSP_FORMAT_RESULT, Some(Box::new(edit)));
}

fn format_json(content: &str, opts: &FormatOpts) -> Result<String, String> {
    let stripped = strip_jsonc_comments(content);
    let mut val: Value = serde_json::from_str(&stripped).map_err(|e| format!("JSON parse error: {e}"))?;
    if opts.sort_keys {
        sort_keys_recursive(&mut val);
    }
    let indent = " ".repeat(opts.indent);
    let mut buf = Vec::new();
    let formatter = PrettyFormatter::with_indent(indent.as_bytes());
    let mut ser = Serializer::with_formatter(&mut buf, formatter);
    val.serialize(&mut ser).map_err(|e| format!("JSON format error: {e}"))?;
    String::from_utf8(buf).map_err(|e| format!("UTF-8 error: {e}"))
}

struct FormatOpts {
    indent: usize,
    sort_keys: bool,
}

fn parse_format_opts(arg: &str) -> FormatOpts {
    let mut opts = FormatOpts {
        indent: 2,
        sort_keys: false,
    };
    let parts: Vec<&str> = arg.split_whitespace().collect();
    let mut i = 0;
    while i < parts.len() {
        match parts[i] {
            "--indent" if i + 1 < parts.len() => {
                opts.indent = parts[i + 1].parse().unwrap_or(2);
                i += 2;
            }
            "--sort-keys" => {
                opts.sort_keys = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    opts
}

fn sort_keys_recursive(val: &mut Value) {
    match val {
        Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                sort_keys_recursive(v);
            }
            let mut entries: Vec<(String, Value)> = map.into_iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            *map = entries.into_iter().collect();
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                sort_keys_recursive(v);
            }
        }
        _ => {}
    }
}

fn strip_jsonc_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            i += 2;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
        } else if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
        } else if bytes[i] == b'"' {
            i = copy_string_literal(bytes, i, &mut out);
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Copy a JSON string literal (including quotes) from `bytes[start..]` into `out`.
/// Returns the index after the closing quote.
fn copy_string_literal(bytes: &[u8], start: usize, out: &mut String) -> usize {
    let len = bytes.len();
    out.push('"');
    let mut i = start + 1;
    while i < len && bytes[i] != b'"' {
        if bytes[i] == b'\\' && i + 1 < len {
            out.push(bytes[i] as char);
            i += 1;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    if i < len {
        out.push('"');
        i += 1;
    }
    i
}

fn extension(path: &Path) -> &str {
    path.extension().and_then(|e| e.to_str()).unwrap_or("")
}
