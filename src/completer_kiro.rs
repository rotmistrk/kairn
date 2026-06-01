//! Kiro command completions: --agent=<name>, --resume, --tui.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use txv_core::complete::CompletionVisitor;

use crate::completer_entry::Entry;

/// Kiro sub-argument completions: --agent=<name>, --resume, --tui.
pub(crate) fn complete_kiro(
    sub: &str,
    root: &Path,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(partial) = sub.strip_prefix("--agent=") {
        return complete_kiro_agents(partial, root, visitor);
    }
    const KIRO_OPTS: &[&str] = &["--agent=", "--resume", "--tui"];
    for o in KIRO_OPTS.iter().filter(|o| o.starts_with(sub)) {
        let e = Entry {
            text: format!("kiro {o}"),
            display: o.to_string(),
            kind: "option",
        };
        if !visitor(&e)? {
            break;
        }
    }
    Ok(())
}

/// Complete agent names by scanning ~/.kiro/agents/ and .kiro/agents/.
fn complete_kiro_agents(
    partial: &str,
    root: &Path,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut names = collect_agent_names(root);
    names.sort();
    for name in names.iter().filter(|n| n.starts_with(partial)) {
        let e = Entry {
            text: format!("kiro --agent={name}"),
            display: name.clone(),
            kind: "agent",
        };
        if !visitor(&e)? {
            break;
        }
    }
    Ok(())
}

fn collect_agent_names(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let home = env::var("HOME").unwrap_or_default();
    let dirs = [PathBuf::from(&home).join(".kiro/agents"), root.join(".kiro/agents")];
    for dir in &dirs {
        scan_agents_dir(dir, &mut names);
    }
    names
}

fn scan_agents_dir(dir: &Path, names: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        if let Some(name) = read_agent_name(&path) {
            if !name.starts_with("kairn-") && !names.contains(&name) {
                names.push(name);
            }
        }
    }
}

fn read_agent_name(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;
    val.get("name").and_then(|n| n.as_str()).map(String::from)
}
