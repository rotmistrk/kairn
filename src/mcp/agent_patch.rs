//! Ensure a kiro agent definition has kairn MCP server configured.
//!
//! On `M-x kiro --agent=foo`, we check `.kiro/agents/foo.json` locally.
//! If it's missing or stale (source in ~/.kiro/agents is newer), we copy
//! the source and patch in the kairn MCP server + allowedTools entry.

use std::env;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use serde_json::{Map, Value};

/// Ensure a patched agent file exists at `.kiro/agents/kairn-<name>.json`.
/// Returns the patched agent name to pass to `--agent=`.
/// For the "kairn" agent itself, returns "kairn" (written by agent_file.rs).
pub fn ensure_agent_patched(root: &Path, agent_name: &str) -> Result<String, String> {
    if agent_name == "kairn" {
        return Ok("kairn".into());
    }

    let source_path = resolve_source_path(agent_name);
    let local_source = root.join(format!(".kiro/agents/{agent_name}.json"));

    // Agent must exist somewhere
    if !local_source.is_file() && source_path.is_none() {
        return Err(format!(
            "agent '{agent_name}' not found in ~/.kiro/agents/ or .kiro/agents/"
        ));
    }

    let patched_name = format!("kairn-{agent_name}");
    let patched_path = root.join(format!(".kiro/agents/{patched_name}.json"));

    if needs_patch(&patched_path, source_path.as_deref().or(Some(&local_source))) {
        let base = load_source(source_path.as_deref(), &local_source, agent_name)?;
        let patched = patch_agent(base)?;
        write_patched(root, &patched_path, &patched)?;
    }
    Ok(patched_name)
}

/// Find the source agent file in ~/.kiro/agents/.
fn resolve_source_path(agent_name: &str) -> Option<std::path::PathBuf> {
    let home = env::var("HOME").ok()?;
    let p = Path::new(&home).join(format!(".kiro/agents/{agent_name}.json"));
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

/// Check if patching is needed: local missing/lacks kairn MCP, or source is newer.
fn needs_patch(local: &Path, source: Option<&Path>) -> bool {
    if !local.is_file() {
        return source.is_some();
    }
    if !local_has_kairn_mcp(local) {
        return true;
    }
    // Check mtime drift: if source is newer than local, re-patch
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
    // No source, no local — create minimal agent with just the name
    Ok(serde_json::json!({"name": agent_name, "tools": ["*"]}))
}

/// Patch the agent JSON to include kairn MCP server and allowedTools.
fn patch_agent(mut val: Value) -> Result<Value, String> {
    let obj = val.as_object_mut().ok_or("agent JSON is not an object")?;

    // Ensure mcpServers.kairn exists
    let servers = obj.entry("mcpServers").or_insert_with(|| Value::Object(Map::new()));
    let servers_obj = servers.as_object_mut().ok_or("mcpServers is not an object")?;
    servers_obj.insert("kairn".to_string(), kairn_mcp_server_def());

    // Ensure allowedTools contains @kairn
    let allowed = obj.entry("allowedTools").or_insert_with(|| Value::Array(Vec::new()));
    if let Some(arr) = allowed.as_array_mut() {
        let tag = Value::String("@kairn".to_string());
        if !arr.contains(&tag) {
            arr.push(tag);
        }
    }

    // Ensure includeMcpJson is true
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
        let result = patch_agent(input).unwrap();
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
        let result = patch_agent(input).unwrap();
        assert!(result["mcpServers"]["other"].is_object());
        assert!(result["mcpServers"]["kairn"].is_object());
        let allowed = result["allowedTools"].as_array().unwrap();
        assert!(allowed.contains(&Value::String("@other".into())));
        assert!(allowed.contains(&Value::String("@kairn".into())));
    }

    #[test]
    fn patch_agent_idempotent() {
        let input: Value = serde_json::json!({"name": "test"});
        let first = patch_agent(input).unwrap();
        let second = patch_agent(first.clone()).unwrap();
        // allowedTools should not have duplicate @kairn
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

        // Override HOME for this test
        let local = root.join(".kiro/agents/myagent.json");
        assert!(!local.exists());

        // Call the internal functions directly (can't override HOME easily)
        let base = load_source(Some(&source), &local, "myagent").unwrap();
        let patched = patch_agent(base).unwrap();
        write_patched(&root, &local, &patched).unwrap();

        assert!(local.exists());
        let content: Value = serde_json::from_str(&fs::read_to_string(&local).unwrap()).unwrap();
        assert!(content["mcpServers"]["kairn"].is_object());
        assert_eq!(content["name"], "myagent");
    }

    #[test]
    fn needs_patch_true_when_local_missing_and_source_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(&source, "{}").unwrap();
        let local = tmp.path().join("nonexistent.json");
        assert!(needs_patch(&local, Some(&source)));
    }

    #[test]
    fn needs_patch_false_when_local_missing_and_no_source() {
        let local = Path::new("/tmp/definitely_nonexistent_kairn_test.json");
        assert!(!needs_patch(local, None));
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
        // Ensure source is newer
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
