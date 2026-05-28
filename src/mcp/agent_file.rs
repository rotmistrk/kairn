//! Write `.kiro/agents/kairn.json` so kiro discovers the MCP server.

use std::env;
use std::fs;
use std::path::Path;

use serde_json::json;

/// Write the agent file at `.kiro/agents/kairn.json` relative to project root.
/// Uses the current executable path. Socket path comes from KAIRN_MCP_SOCKET env var at runtime.
pub fn write_agent_file(root: &Path) {
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
