//! Tcl commands for LSP configuration.
//!
//! Registers `lsp-server` and `lsp-disable` commands that store
//! their arguments in interpreter variables for later extraction.

use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;

use super::registry::LspRegistry;

const KNOWN_LANGS: &[&str] = &[
    "rust",
    "go",
    "typescript",
    "javascript",
    "c",
    "cpp",
    "java",
    "python",
    "ruby",
    "lua",
];

/// Register LSP-related Tcl commands on the interpreter.
pub fn register_lsp_commands(interp: &mut Interpreter) {
    interp.register_fn("lsp-server", |interp, args| {
        if args.len() < 2 {
            return Err(rusticle::error::TclError::new(
                "wrong # args: should be \"lsp-server lang cmd ?args?\"",
            ));
        }
        let lang = args[0].as_str().to_string();
        let cmd_and_args: Vec<String> = args[1..].iter().map(|a| a.as_str().to_string()).collect();
        let val = cmd_and_args.join(" ");
        let var_name = format!("lsp.server.{lang}");
        interp.set_var(&var_name, TclValue::from(val));
        Ok(TclValue::Str(String::new()))
    });

    interp.register_fn("lsp-disable", |interp, args| {
        if args.is_empty() {
            return Err(rusticle::error::TclError::new(
                "wrong # args: should be \"lsp-disable lang\"",
            ));
        }
        let lang = args[0].as_str().to_string();
        let var_name = format!("lsp.disable.{lang}");
        interp.set_var(&var_name, TclValue::from("1"));
        Ok(TclValue::Str(String::new()))
    });
}

/// Extract LSP configuration from interpreter variables into the registry.
pub fn apply_lsp_config(interp: &Interpreter, registry: &mut LspRegistry) {
    for lang in KNOWN_LANGS {
        let server_var = format!("lsp.server.{lang}");
        if let Some(val) = interp.get_var(&server_var) {
            let s = val.as_str();
            let parts: Vec<&str> = s.splitn(2, ' ').collect();
            let cmd = parts[0];
            let args: Vec<String> = if parts.len() > 1 {
                parts[1].split_whitespace().map(|a| a.to_string()).collect()
            } else {
                Vec::new()
            };
            registry.set_config(lang, cmd, &args);
        }
        let disable_var = format!("lsp.disable.{lang}");
        if interp.get_var(&disable_var).is_some() {
            registry.disable(lang);
        }
    }
}

/// Format LSP status for display.
pub fn format_lsp_status(registry: &LspRegistry) -> String {
    let active = registry.active_languages();
    if active.is_empty() {
        "LSP: no active servers".to_string()
    } else {
        format!("LSP active: {}", active.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_extract_server() {
        let mut interp = Interpreter::new();
        register_lsp_commands(&mut interp);
        interp.eval("lsp-server rust my-analyzer --flag").unwrap();

        let mut reg = LspRegistry::new();
        apply_lsp_config(&interp, &mut reg);
        assert!(reg.has_config("rust"));
    }

    #[test]
    fn register_and_extract_disable() {
        let mut interp = Interpreter::new();
        register_lsp_commands(&mut interp);
        interp.eval("lsp-disable python").unwrap();

        let mut reg = LspRegistry::new();
        apply_lsp_config(&interp, &mut reg);
        let result = reg.get_or_start("python", std::path::Path::new("/tmp"));
        assert!(result.is_none());
    }

    #[test]
    fn format_status_empty() {
        let reg = LspRegistry::new();
        let status = format_lsp_status(&reg);
        assert!(status.contains("no active"));
    }
}
