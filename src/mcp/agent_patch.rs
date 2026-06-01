//! Ensure a kiro agent definition has kairn MCP server configured.
//!
//! On `M-x kiro --agent=foo`, we check `.kiro/agents/foo.json` locally.
//! If it's missing or stale (source in ~/.kiro/agents is newer), we copy
//! the source and patch in the kairn MCP server + allowedTools entry.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde_json::{Map, Value};

/// Ensure a patched agent file exists at `.kiro/agents/kairn-<name>.json`.
/// Returns the patched agent name to pass to `--agent=`.
/// For the "kairn" agent itself, returns "kairn" (written by agent_file.rs).
pub fn ensure_agent_patched(root: &Path, agent_name: &str) -> Result<String, String> {
    if agent_name == "kairn" {
        return Ok("kairn".into());
    }

    let home = env::var("HOME").unwrap_or_default();
    let home_dir = Path::new(&home).join(".kiro/agents");
    let source_path = find_agent_by_name(&home_dir, agent_name);
    let local_source = find_agent_by_name(&root.join(".kiro/agents"), agent_name);

    if local_source.is_none() && source_path.is_none() {
        return Err(format!(
            "agent '{agent_name}' not found in ~/.kiro/agents/ or .kiro/agents/"
        ));
    }

    let patched_name = format!("kairn-{agent_name}");
    let patched_path = root.join(format!(".kiro/agents/{patched_name}.json"));
    let best_source = source_path.as_deref().or(local_source.as_deref());

    if needs_patch(&patched_path, best_source) {
        let base = load_source(best_source, &patched_path, agent_name)?;
        let patched = patch_agent(base, &patched_name)?;
        write_patched(root, &patched_path, &patched)?;
    }
    Ok(patched_name)
}

/// Scan a directory for an agent JSON matching by filename or "name" field.
fn find_agent_by_name(dir: &Path, agent_name: &str) -> Option<PathBuf> {
    let exact = dir.join(format!("{agent_name}.json"));
    if exact.is_file() {
        return Some(exact);
    }
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if agent_json_has_name(&path, agent_name) {
            return Some(path);
        }
    }
    None
}

fn agent_json_has_name(path: &Path, agent_name: &str) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<Value>(&content) else {
        return false;
    };
    val.get("name").and_then(|n| n.as_str()) == Some(agent_name)
}

/// Check if patching is needed: local missing/lacks kairn MCP, or source is newer.
fn needs_patch(local: &Path, source: Option<&Path>) -> bool {
    if !local.is_file() {
        return source.is_some();
    }
    if !local_has_kairn_mcp(local) {
        return true;
    }
    let Some(source) = source else {
        return false;
    };
    let Ok(local_mtime) = mtime(local) else {
        return true;
    };
    let Ok(source_mtime) = mtime(source) else {
        return false;
    };
    source_mtime > local_mtime
}

fn local_has_kairn_mcp(path: &Path) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(val) = serde_json::from_str::<Value>(&content) else {
        return false;
    };
    val.get("mcpServers").and_then(|s| s.get("kairn")).is_some()
}

fn mtime(path: &Path) -> Result<SystemTime, std::io::Error> {
    fs::metadata(path)?.modified()
}

/// Load the base agent JSON from source, existing local, or create minimal.
fn load_source(source: Option<&Path>, local: &Path, agent_name: &str) -> Result<Value, String> {
    if let Some(path) = source {
        let content = fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?;
        return serde_json::from_str(&content).map_err(|e| format!("parse {}: {e}", path.display()));
    }
    if local.is_file() {
        let content = fs::read_to_string(local).map_err(|e| format!("read {}: {e}", local.display()))?;
        return serde_json::from_str(&content).map_err(|e| format!("parse {}: {e}", local.display()));
    }
    Ok(serde_json::json!({"name": agent_name, "tools": ["*"]}))
}

/// Patch the agent JSON to include kairn MCP server and allowedTools.
fn patch_agent(mut val: Value, patched_name: &str) -> Result<Value, String> {
    let obj = val.as_object_mut().ok_or("agent JSON is not an object")?;
    obj.insert("name".to_string(), Value::String(patched_name.to_string()));
    let servers = obj.entry("mcpServers").or_insert_with(|| Value::Object(Map::new()));
    let servers_obj = servers.as_object_mut().ok_or("mcpServers is not an object")?;
    servers_obj.insert("kairn".to_string(), kairn_mcp_server_def());

    let allowed = obj.entry("allowedTools").or_insert_with(|| Value::Array(Vec::new()));
    if let Some(arr) = allowed.as_array_mut() {
        let tag = Value::String("@kairn".to_string());
        if !arr.contains(&tag) {
            arr.push(tag);
        }
    }
    obj.insert("includeMcpJson".to_string(), Value::Bool(true));
    Ok(val)
}

fn kairn_mcp_server_def() -> Value {
    let bin = env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "kairn".to_owned());
    let socket = env::var("KAIRN_MCP_SOCKET").unwrap_or_default();
    serde_json::json!({
        "command": bin,
        "args": ["--mcp-connect"],
        "env": {"KAIRN_MCP_SOCKET": socket}
    })
}

fn write_patched(root: &Path, local: &Path, val: &Value) -> Result<(), String> {
    let dir = root.join(".kiro/agents");
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let json = serde_json::to_string_pretty(val).map_err(|e| format!("serialize: {e}"))?;
    fs::write(local, json).map_err(|e| format!("write {}: {e}", local.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn patch_agent_adds_kairn_mcp_to_empty_agent() {
        let input: Value = serde_json::json!({"name": "test", "tools": ["*"]});
        let result = patch_agent(input, "kairn-test").unwrap();
        assert_eq!(result["name"], "kairn-test");
        assert!(result["mcpServers"]["kairn"].is_object());
        assert_eq!(result["mcpServers"]["kairn"]["args"][0], "--mcp-connect");
        assert!(result["allowedTools"]
            .as_array()
            .unwrap()
            .contains(&Value::String("@kairn".into())));
        assert_eq!(result["includeMcpJson"], true);
    }

    #[test]
    fn patch_agent_preserves_existing_servers() {
        let input: Value = serde_json::json!({
            "name": "test",
            "mcpServers": {"other": {"command": "foo"}},
            "allowedTools": ["@other"]
        });
        let result = patch_agent(input, "kairn-test").unwrap();
        assert!(result["mcpServers"]["other"].is_object());
        assert!(result["mcpServers"]["kairn"].is_object());
        let allowed = result["allowedTools"].as_array().unwrap();
        assert!(allowed.contains(&Value::String("@other".into())));
        assert!(allowed.contains(&Value::String("@kairn".into())));
    }

    #[test]
    fn patch_agent_idempotent() {
        let input: Value = serde_json::json!({"name": "test"});
        let first = patch_agent(input, "kairn-test").unwrap();
        let second = patch_agent(first.clone(), "kairn-test").unwrap();
        let allowed = second["allowedTools"].as_array().unwrap();
        let kairn_count = allowed.iter().filter(|v| v.as_str() == Some("@kairn")).count();
        assert_eq!(kairn_count, 1);
    }

    #[test]
    fn ensure_agent_patched_creates_local_from_source() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("project");
        let source_dir = tmp.path().join("home/.kiro/agents");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&root).unwrap();

        let source = source_dir.join("myagent.json");
        fs::write(&source, r#"{"name":"myagent","tools":["*"]}"#).unwrap();
        let local = root.join(".kiro/agents/myagent.json");

        let base = load_source(Some(&source), &local, "myagent").unwrap();
        let patched = patch_agent(base, "kairn-test").unwrap();
        write_patched(&root, &local, &patched).unwrap();

        assert!(local.exists());
        let content: Value = serde_json::from_str(&fs::read_to_string(&local).unwrap()).unwrap();
        assert!(content["mcpServers"]["kairn"].is_object());
        assert_eq!(content["name"], "kairn-test");
    }

    #[test]
    fn needs_patch_true_when_local_missing_and_source_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(&source, "{}").unwrap();
        assert!(needs_patch(&tmp.path().join("nonexistent.json"), Some(&source)));
    }

    #[test]
    fn needs_patch_false_when_local_missing_and_no_source() {
        assert!(!needs_patch(
            Path::new("/tmp/definitely_nonexistent_kairn_test.json"),
            None
        ));
    }

    #[test]
    fn needs_patch_true_when_local_lacks_kairn_mcp() {
        let tmp = tempfile::tempdir().unwrap();
        let local = tmp.path().join("agent.json");
        fs::write(&local, r#"{"name":"test"}"#).unwrap();
        assert!(needs_patch(&local, None));
    }

    #[test]
    fn needs_patch_false_when_local_has_kairn_and_no_source() {
        let tmp = tempfile::tempdir().unwrap();
        let local = tmp.path().join("agent.json");
        fs::write(&local, r#"{"mcpServers":{"kairn":{"command":"x"}}}"#).unwrap();
        assert!(!needs_patch(&local, None));
    }

    #[test]
    fn needs_patch_true_when_source_newer() {
        let tmp = tempfile::tempdir().unwrap();
        let local = tmp.path().join("local.json");
        let source = tmp.path().join("source.json");
        fs::write(&local, r#"{"mcpServers":{"kairn":{"command":"x"}}}"#).unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(&source, r#"{"name":"updated"}"#).unwrap();
        assert!(needs_patch(&local, Some(&source)));
    }

    #[test]
    fn needs_patch_false_when_local_newer() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source.json");
        let local = tmp.path().join("local.json");
        fs::write(&source, r#"{"name":"old"}"#).unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(&local, r#"{"mcpServers":{"kairn":{"command":"x"}}}"#).unwrap();
        assert!(!needs_patch(&local, Some(&source)));
    }
}
