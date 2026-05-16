//! M-x `lsp` subcommand dispatch: start, restart, stop, timeout, args.

use crate::handler::AppState;

/// Parse and execute `lsp <subcommand> <glob> [value]`. Returns status message.
pub fn handle_lsp_command(arg: &str, state: &mut AppState) -> String {
    let parts: Vec<&str> = arg.splitn(3, ' ').collect();
    let sub = parts.first().copied().unwrap_or("");
    let pattern = parts.get(1).copied().unwrap_or("*");
    let value = parts.get(2).copied().unwrap_or("");

    let result = match sub {
        "start" => lsp_start(pattern, state),
        "restart" => lsp_restart(pattern, state),
        "stop" => lsp_stop(pattern, state),
        "timeout" => lsp_timeout(pattern, value, state),
        "args" => lsp_args(pattern, value, state),
        "status" => crate::lsp::config_commands::format_lsp_status(&state.lsp),
        _ => format!("lsp: unknown subcommand '{sub}' (start|restart|stop|timeout|args|status)"),
    };
    refresh_lsp_languages(state);
    result
}

/// Sync the shared language list from registry configs + active servers.
pub fn refresh_lsp_languages(state: &AppState) {
    let langs = state.lsp.matching_languages("*");
    if let Ok(mut guard) = state.lsp_languages.lock() {
        *guard = langs;
    }
}

fn lsp_start(pattern: &str, state: &mut AppState) -> String {
    let langs = state.lsp.matching_languages(pattern);
    if langs.is_empty() {
        return format!("No languages matching '{pattern}'");
    }
    let root = state.root_dir.clone();
    let mut started = Vec::new();
    for lang in &langs {
        if state.lsp.get_or_start(lang, &root).is_some() {
            started.push(lang.as_str());
        }
    }
    if started.is_empty() {
        "No servers started (check config)".to_string()
    } else {
        format!("LSP started: {}", started.join(", "))
    }
}

fn lsp_restart(pattern: &str, state: &mut AppState) -> String {
    let langs = state.lsp.matching_languages(pattern);
    if langs.is_empty() {
        return format!("No languages matching '{pattern}'");
    }
    for lang in &langs {
        state.lsp.restart(lang);
        state.lsp_status.remove(lang);
    }
    let root = state.root_dir.clone();
    let mut restarted = Vec::new();
    for lang in &langs {
        if state.lsp.get_or_start(lang, &root).is_some() {
            restarted.push(lang.as_str());
        }
    }
    format!("LSP restarted: {}", restarted.join(", "))
}

fn lsp_stop(pattern: &str, state: &mut AppState) -> String {
    let langs = state.lsp.matching_languages(pattern);
    if langs.is_empty() {
        return format!("No languages matching '{pattern}'");
    }
    let mut stopped = Vec::new();
    for lang in &langs {
        if state.lsp.stop(lang) {
            state.lsp_status.remove(lang);
            stopped.push(lang.as_str());
        }
    }
    if stopped.is_empty() {
        "No active servers to stop".to_string()
    } else {
        format!("LSP stopped: {}", stopped.join(", "))
    }
}

fn lsp_timeout(pattern: &str, value: &str, state: &mut AppState) -> String {
    let langs = state.lsp.matching_languages(pattern);
    if langs.is_empty() {
        return format!("No languages matching '{pattern}'");
    }
    if value.is_empty() {
        // Query current timeout
        let info: Vec<String> = langs
            .iter()
            .map(|l| {
                let t = state.lsp.timeout(l).unwrap_or(state.lsp_pending.timeout_secs);
                format!("{l}: {t}s")
            })
            .collect();
        return info.join(", ");
    }
    let Ok(secs) = value.parse::<u64>() else {
        return format!("Invalid timeout: '{value}' (expected seconds)");
    };
    for lang in &langs {
        state.lsp.set_timeout(lang, secs);
    }
    format!("Timeout set to {secs}s for: {}", langs.join(", "))
}

fn lsp_args(pattern: &str, value: &str, state: &mut AppState) -> String {
    let langs = state.lsp.matching_languages(pattern);
    if langs.is_empty() {
        return format!("No languages matching '{pattern}'");
    }
    if value.is_empty() {
        return "Usage: lsp args <lang> <command> [args...]".to_string();
    }
    let parts: Vec<&str> = value.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args: Vec<String> = if parts.len() > 1 {
        parts[1].split_whitespace().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };
    for lang in &langs {
        state.lsp.set_config(lang, cmd, &args);
    }
    format!("Config updated for: {}", langs.join(", "))
}
