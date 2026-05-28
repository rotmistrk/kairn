//! `--init-home` and `--init-wp` commands: write default configs or show diffs.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const HOME_CONFIG_TEMPLATE: &str = include_str!("../doc/example-init.tcl");

const WP_CONFIG_TEMPLATE: &str = "\
# kairn project config — .kairn/init.tcl
# Tcl syntax. Overrides ~/.config/kairn/init.tcl for this project.

# ─── Build Commands ─────────────────────────
# Auto-detected from Cargo.toml / Makefile / package.json.
# Override in .kairn/init (plain text, not Tcl):
#   build = make -j8
#   test = make check
#   test-file = cargo test --lib {file}

# ─── Editor ─────────────────────────────────
# set editor.tabstop 4

# ─── Hooks ──────────────────────────────────
# hook add file-save { build run }
# keymap bind ctrl+b { build run }

# ─── LSP ──────────────────────────────────
# lsp rust-analyzer {
#     command \"rust-analyzer\"
#     filetypes {rs}
# }
";

pub fn init_home_config() -> anyhow::Result<()> {
    let path = config_dir().join("init.tcl");
    write_or_diff(&path, HOME_CONFIG_TEMPLATE, "home")
}

pub fn init_wp_config(project: &Path) -> anyhow::Result<()> {
    let path = project.join(".kairn").join("init.tcl");
    write_or_diff(&path, WP_CONFIG_TEMPLATE, "project")?;
    write_agent_config(project)?;
    Ok(())
}

/// Write .kiro/agents/kairn.json with the correct local binary path.
fn write_agent_config(project: &Path) -> anyhow::Result<()> {
    let agent_path = project.join(".kiro/agents/kairn.json");
    if agent_path.exists() {
        println!("agent config already exists: {}", agent_path.display());
        return Ok(());
    }
    let json = r#"{
  "name": "kairn",
  "tools": ["*"],
  "allowedTools": ["@kairn"],
  "includeMcpJson": true,
  "mcpServers": {
    "kairn": {
      "command": "kairn",
      "args": ["--mcp-connect"],
      "env": {
        "KAIRN_MCP_SOCKET": "${KAIRN_MCP_SOCKET}"
      }
    }
  }
}
"#;
    if let Some(parent) = agent_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&agent_path, json)?;
    println!("Created agent config: {}", agent_path.display());
    Ok(())
}

fn write_or_diff(path: &Path, template: &str, label: &str) -> anyhow::Result<()> {
    if path.exists() {
        let existing = fs::read_to_string(path)?;
        let missing = find_missing_settings(template, &existing);
        if missing.is_empty() {
            println!("{} config is up to date: {}", label, path.display());
        } else {
            println!(
                "{} config exists: {}\n\nNew settings available (copy what you need):\n",
                label,
                path.display()
            );
            for line in &missing {
                println!("{line}");
            }
        }
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, template)?;
        println!("Created {} config: {}", label, path.display());
    }
    Ok(())
}

/// Find `set` lines from template not present in existing config.
fn find_missing_settings(template: &str, existing: &str) -> Vec<String> {
    let existing_keys: HashSet<&str> = existing
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            let check = t.strip_prefix("# ").unwrap_or(t);
            extract_set_key(check)
        })
        .collect();

    let mut missing = Vec::new();
    let mut context_comment = String::new();
    for line in template.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ───") {
            context_comment = line.to_string();
            continue;
        }
        let check = trimmed.strip_prefix("# ").unwrap_or(trimmed);
        let Some(key) = extract_set_key(check) else {
            continue;
        };
        if existing_keys.contains(key) {
            continue;
        }
        if !context_comment.is_empty() {
            if !missing.is_empty() {
                missing.push(String::new());
            }
            missing.push(context_comment.clone());
            context_comment.clear();
        }
        missing.push(line.to_string());
    }
    missing
}

fn extract_set_key(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("set ")?;
    rest.split_whitespace().next()
}

fn config_dir() -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        })
        .join("kairn")
}
