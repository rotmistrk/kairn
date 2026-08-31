//! Write `.kiro/agents/kairn.json` so kiro discovers the MCP server.

use std::env;
use std::fs;
use std::path::Path;

use serde_json::json;

/// Steering doc template for kairn agent.
const STEERING_TEMPLATE: &str = include_str!("../../doc/kairn-agent-sop.md");

/// Write the agent file at `.kiro/agents/kairn.json` relative to project root.
/// Uses the current executable path. Socket path comes from KAIRN_MCP_SOCKET env var at runtime.
pub fn write_agent_file(root: &Path) {
    write_agent_json(root);
    write_steering_doc(root);
}

fn write_agent_json(root: &Path) {
    let agents_dir = root.join(".kiro/agents");
    if let Err(e) = fs::create_dir_all(&agents_dir) {
        log::error!("MCP agent: create_dir_all {}: {e}", agents_dir.display());
        return;
    }

    let bin = env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "kairn".to_owned());

    let config = json!({
        "name": "kairn",
        "mcpServers": {
            "kairn": {
                "command": bin,
                "args": ["--mcp-connect"],
                "env": {"KAIRN_MCP_SOCKET": "${KAIRN_MCP_SOCKET}"}
            }
        },
        "includeMcpJson": true,
        "tools": ["*"],
        "allowedTools": ["@kairn"]
    });

    let json = serde_json::to_string_pretty(&config).unwrap_or_default();
    if let Err(e) = fs::write(agents_dir.join("kairn.json"), &json) {
        log::error!("MCP agent: write kairn.json: {e}");
    }
}

fn write_steering_doc(root: &Path) {
    let steering_dir = root.join(".kiro/steering");
    let steering_path = steering_dir.join("kairn-agent-sop.md");

    // Don't overwrite if it already exists (user may have customized it)
    if steering_path.exists() {
        return;
    }

    if let Err(e) = fs::create_dir_all(&steering_dir) {
        log::error!("MCP agent: create_dir_all {}: {e}", steering_dir.display());
        return;
    }

    if let Err(e) = fs::write(&steering_path, STEERING_TEMPLATE) {
        log::error!("MCP agent: write kairn-agent-sop.md: {e}");
    }
}
