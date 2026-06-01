#![cfg(test)]

use super::*;
use std::thread;
use std::time::Duration;

type R = Result<(), Box<dyn std::error::Error>>;

#[test]
fn patch_agent_adds_kairn_mcp_to_empty_agent() -> R {
    let input: Value = serde_json::json!({"name": "test", "tools": ["*"]});
    let result = patch_agent(input, "kairn-test")?;
    assert_eq!(result["name"], "kairn-test");
    assert!(result["mcpServers"]["kairn"].is_object());
    assert_eq!(result["mcpServers"]["kairn"]["args"][0], "--mcp-connect");
    let allowed = result["allowedTools"].as_array().ok_or("no allowedTools")?;
    assert!(allowed.contains(&Value::String("@kairn".into())));
    let tools = result["tools"].as_array().ok_or("no tools")?;
    assert!(tools.contains(&Value::String("@kairn".into())));
    assert_eq!(result["includeMcpJson"], true);
    Ok(())
}

#[test]
fn patch_agent_preserves_existing_servers() -> R {
    let input: Value = serde_json::json!({
        "name": "test",
        "mcpServers": {"other": {"command": "foo"}},
        "allowedTools": ["@other"],
        "tools": ["@other"]
    });
    let result = patch_agent(input, "kairn-test")?;
    assert!(result["mcpServers"]["other"].is_object());
    assert!(result["mcpServers"]["kairn"].is_object());
    let allowed = result["allowedTools"].as_array().ok_or("no allowedTools")?;
    assert!(allowed.contains(&Value::String("@other".into())));
    assert!(allowed.contains(&Value::String("@kairn".into())));
    let tools = result["tools"].as_array().ok_or("no tools")?;
    assert!(tools.contains(&Value::String("@other".into())));
    assert!(tools.contains(&Value::String("@kairn".into())));
    Ok(())
}

#[test]
fn patch_agent_idempotent() -> R {
    let input: Value = serde_json::json!({"name": "test"});
    let first = patch_agent(input, "kairn-test")?;
    let second = patch_agent(first.clone(), "kairn-test")?;
    let allowed = second["allowedTools"].as_array().ok_or("no allowedTools")?;
    let kairn_count = allowed.iter().filter(|v| v.as_str() == Some("@kairn")).count();
    assert_eq!(kairn_count, 1);
    Ok(())
}

#[test]
fn ensure_agent_patched_creates_local_from_source() -> R {
    let tmp = tempfile::tempdir()?;
    let root = tmp.path().join("project");
    let source_dir = tmp.path().join("home/.kiro/agents");
    fs::create_dir_all(&source_dir)?;
    fs::create_dir_all(&root)?;
    let source = source_dir.join("myagent.json");
    fs::write(&source, r#"{"name":"myagent","tools":["*"]}"#)?;
    let local = root.join(".kiro/agents/myagent.json");
    let base = load_source(Some(&source), &local, "myagent")?;
    let patched = patch_agent(base, "kairn-test")?;
    write_patched(&root, &local, &patched)?;
    assert!(local.exists());
    let content: Value = serde_json::from_str(&fs::read_to_string(&local)?)?;
    assert!(content["mcpServers"]["kairn"].is_object());
    assert_eq!(content["name"], "kairn-test");
    Ok(())
}

#[test]
fn needs_patch_true_when_local_missing_and_source_exists() -> R {
    let tmp = tempfile::tempdir()?;
    let source = tmp.path().join("source.json");
    fs::write(&source, "{}")?;
    assert!(needs_patch(&tmp.path().join("nonexistent.json"), Some(&source)));
    Ok(())
}

#[test]
fn needs_patch_false_when_local_missing_and_no_source() {
    assert!(!needs_patch(
        Path::new("/tmp/definitely_nonexistent_kairn_test.json"),
        None
    ));
}

#[test]
fn needs_patch_true_when_local_lacks_kairn_mcp() -> R {
    let tmp = tempfile::tempdir()?;
    let local = tmp.path().join("agent.json");
    fs::write(&local, r#"{"name":"test"}"#)?;
    assert!(needs_patch(&local, None));
    Ok(())
}

#[test]
fn needs_patch_false_when_local_has_kairn_and_no_source() -> R {
    let tmp = tempfile::tempdir()?;
    let local = tmp.path().join("agent.json");
    fs::write(&local, r#"{"mcpServers":{"kairn":{"command":"x"}}}"#)?;
    assert!(!needs_patch(&local, None));
    Ok(())
}

#[test]
fn needs_patch_true_when_source_newer() -> R {
    let tmp = tempfile::tempdir()?;
    let local = tmp.path().join("local.json");
    let source = tmp.path().join("source.json");
    fs::write(&local, r#"{"mcpServers":{"kairn":{"command":"x"}}}"#)?;
    thread::sleep(Duration::from_millis(50));
    fs::write(&source, r#"{"name":"updated"}"#)?;
    assert!(needs_patch(&local, Some(&source)));
    Ok(())
}

#[test]
fn needs_patch_false_when_local_newer() -> R {
    let tmp = tempfile::tempdir()?;
    let source = tmp.path().join("source.json");
    let local = tmp.path().join("local.json");
    fs::write(&source, r#"{"name":"old"}"#)?;
    thread::sleep(Duration::from_millis(50));
    fs::write(&local, r#"{"mcpServers":{"kairn":{"command":"x"}}}"#)?;
    assert!(!needs_patch(&local, Some(&source)));
    Ok(())
}
