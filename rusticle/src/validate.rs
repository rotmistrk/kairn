//! Load-time validation pass: static analysis for scripts.

use std::collections::{HashMap, HashSet};

use crate::error::Location;
use crate::interpreter::Interpreter;
use crate::parser::{Command, Parser};

/// Severity of a diagnostic.
#[derive(Clone, Debug, PartialEq)]
pub enum Severity {
    /// An error that prevents execution.
    Error,
    /// A warning that may indicate a problem.
    Warning,
}

/// A single diagnostic from validation.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// Human-readable message.
    pub message: String,
    /// Source location.
    pub location: Location,
    /// Severity level.
    pub severity: Severity,
    /// Optional suggestion for fixing.
    pub suggestion: Option<String>,
}

/// Result of validating a script.
#[derive(Clone, Debug, Default)]
pub struct ValidationResult {
    /// Errors found.
    pub errors: Vec<Diagnostic>,
    /// Warnings found.
    pub warnings: Vec<Diagnostic>,
}

impl ValidationResult {
    /// Returns true if there are no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    fn add_error(&mut self, msg: String, line: usize, suggestion: Option<String>) {
        self.errors.push(Diagnostic {
            message: msg,
            location: Location {
                source: String::new(),
                line,
                col: 0,
            },
            severity: Severity::Error,
            suggestion,
        });
    }

    fn add_warning(&mut self, msg: String, line: usize) {
        self.warnings.push(Diagnostic {
            message: msg,
            location: Location {
                source: String::new(),
                line,
                col: 0,
            },
            severity: Severity::Warning,
            suggestion: None,
        });
    }
}

/// Validation context tracking known symbols.
struct ValidationContext {
    /// Known command names (builtins + procs).
    known_commands: HashSet<String>,
    /// Known variable names per scope.
    known_vars: Vec<HashSet<String>>,
    /// Defined procs with their arity (min, max).
    proc_arities: HashMap<String, (usize, usize)>,
    /// Called procs (for dead code detection).
    called_procs: HashSet<String>,
    /// Defined procs (for dead code detection).
    defined_procs: HashSet<String>,
}

impl ValidationContext {
    fn new(interp: &Interpreter) -> Self {
        let mut known_commands: HashSet<String> = interp.commands.keys().cloned().collect();
        known_commands.extend(interp.procs.keys().cloned());
        // Add common structural commands
        for cmd in &["context", "declare", "manifest", "try", "on", "finally"] {
            known_commands.insert((*cmd).to_string());
        }
        let mut proc_arities = HashMap::new();
        for (name, proc_def) in &interp.procs {
            let min = proc_def
                .params
                .iter()
                .filter(|p| p.default.is_none())
                .count();
            let max = proc_def.params.len();
            proc_arities.insert(name.clone(), (min, max));
        }
        Self {
            known_commands,
            known_vars: vec![HashSet::new()],
            proc_arities,
            called_procs: HashSet::new(),
            defined_procs: HashSet::new(),
        }
    }

    fn push_scope(&mut self) {
        self.known_vars.push(HashSet::new());
    }

    fn pop_scope(&mut self) {
        if self.known_vars.len() > 1 {
            self.known_vars.pop();
        }
    }

    fn define_var(&mut self, name: &str) {
        if let Some(scope) = self.known_vars.last_mut() {
            scope.insert(name.to_string());
        }
    }

    fn is_var_known(&self, name: &str) -> bool {
        self.known_vars.iter().any(|s| s.contains(name))
    }

    fn is_var_in_current_scope(&self, name: &str) -> bool {
        self.known_vars
            .last()
            .map(|s| s.contains(name))
            .unwrap_or(false)
    }
}

impl Interpreter {
    /// Validate a script without executing it.
    pub fn validate(&self, script: &str) -> ValidationResult {
        let mut result = ValidationResult::default();
        let parsed = match Parser::parse(script) {
            Ok(p) => p,
            Err(e) => {
                result.add_error(e.message, 0, None);
                return result;
            }
        };
        let mut ctx = ValidationContext::new(self);
        validate_commands(&parsed.commands, &mut ctx, &mut result);
        check_dead_code(&ctx, &mut result);
        result
    }
}

/// Validate a list of commands.
fn validate_commands(
    commands: &[Command],
    ctx: &mut ValidationContext,
    result: &mut ValidationResult,
) {
    let mut after_return = false;
    for cmd in commands {
        if cmd.words.is_empty() {
            continue;
        }
        if after_return {
            result.add_warning("unreachable code after return".into(), cmd.line);
        }
        let name = cmd.words[0].text().to_string();
        validate_one_command(&name, cmd, ctx, result);
        if name == "return" {
            after_return = true;
        }
    }
}

/// Validate a single command.
fn validate_one_command(
    name: &str,
    cmd: &Command,
    ctx: &mut ValidationContext,
    result: &mut ValidationResult,
) {
    match name {
        "set" => validate_set(cmd, ctx, result),
        "proc" => validate_proc(cmd, ctx, result),
        "if" | "while" | "foreach" | "for" => {
            validate_body_words(cmd, ctx, result);
        }
        "switch" => validate_switch(cmd, ctx, result),
        _ => {
            check_command_exists(name, cmd, ctx, result);
            check_proc_arity(name, cmd, ctx, result);
            check_var_references(cmd, ctx, result);
            ctx.called_procs.insert(name.to_string());
        }
    }
}

/// Validate a `set` command.
fn validate_set(cmd: &Command, ctx: &mut ValidationContext, result: &mut ValidationResult) {
    if cmd.words.len() >= 2 {
        let var_name = cmd.words[1].text().to_string();
        // Check for shadowing
        if ctx.is_var_known(&var_name) && !ctx.is_var_in_current_scope(&var_name) {
            result.add_warning(
                format!("variable \"{var_name}\" shadows outer variable"),
                cmd.line,
            );
        }
        ctx.define_var(&var_name);
    }
    check_var_references(cmd, ctx, result);
}

/// Validate a `proc` definition.
fn validate_proc(cmd: &Command, ctx: &mut ValidationContext, result: &mut ValidationResult) {
    if cmd.words.len() >= 4 {
        let proc_name = cmd.words[1].text().to_string();
        let params = cmd.words[2].text();
        let body = cmd.words[3].text();

        ctx.defined_procs.insert(proc_name.clone());
        ctx.known_commands.insert(proc_name.clone());

        // Count params for arity
        let param_count = if params.trim().is_empty() {
            0
        } else {
            params.split_whitespace().count()
        };
        ctx.proc_arities
            .insert(proc_name, (param_count, param_count));

        // Validate body
        ctx.push_scope();
        for p in params.split_whitespace() {
            let name = p.trim_matches(|c| c == '{' || c == '}');
            let name = name.split(':').next().unwrap_or(name);
            ctx.define_var(name);
        }
        if let Ok(parsed) = Parser::parse(body) {
            validate_commands(&parsed.commands, ctx, result);
        }
        ctx.pop_scope();
    }
}

/// Validate body words (for if/while/foreach/for).
fn validate_body_words(cmd: &Command, ctx: &mut ValidationContext, result: &mut ValidationResult) {
    for word in &cmd.words[1..] {
        let text = word.text();
        // Try to parse as a script body
        if let Ok(parsed) = Parser::parse(text) {
            if !parsed.commands.is_empty() {
                ctx.push_scope();
                validate_commands(&parsed.commands, ctx, result);
                ctx.pop_scope();
            }
        }
    }
    check_var_references(cmd, ctx, result);
}

/// Validate a switch command for non-exhaustive patterns.
fn validate_switch(cmd: &Command, ctx: &mut ValidationContext, result: &mut ValidationResult) {
    if cmd.words.len() >= 3 {
        let body = cmd.words[2].text();
        let has_default = body.contains("default") || body.contains('_');
        if !has_default {
            result.add_warning(
                "switch may not be exhaustive (no default case)".into(),
                cmd.line,
            );
        }
    }
    check_var_references(cmd, ctx, result);
}

/// Check if a command name is known.
fn check_command_exists(
    name: &str,
    cmd: &Command,
    ctx: &ValidationContext,
    result: &mut ValidationResult,
) {
    if ctx.known_commands.contains(name) {
        return;
    }
    // "Did you mean?" suggestion
    let suggestion = find_similar(name, &ctx.known_commands);
    result.add_error(format!("unknown command \"{name}\""), cmd.line, suggestion);
}

/// Check proc arity.
fn check_proc_arity(
    name: &str,
    cmd: &Command,
    ctx: &ValidationContext,
    result: &mut ValidationResult,
) {
    if let Some((min, max)) = ctx.proc_arities.get(name) {
        let arg_count = cmd.words.len() - 1;
        if arg_count < *min || arg_count > *max {
            result.add_error(
                format!("\"{name}\" expects {min}..{max} args, got {arg_count}"),
                cmd.line,
                None,
            );
        }
    }
}

/// Check variable references in a command for undefined variables.
fn check_var_references(cmd: &Command, ctx: &ValidationContext, result: &mut ValidationResult) {
    for word in &cmd.words {
        let text = word.text();
        check_vars_in_text(text, cmd.line, ctx, result);
    }
}

/// Scan text for $var references and check if they're defined.
fn check_vars_in_text(
    text: &str,
    line: usize,
    ctx: &ValidationContext,
    result: &mut ValidationResult,
) {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '$' {
            i += 1;
            let mut name = String::new();
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == ':')
            {
                name.push(chars[i]);
                i += 1;
            }
            if !name.is_empty() && !ctx.is_var_known(&name) {
                result.add_warning(format!("possibly undefined variable \"{name}\""), line);
            }
        } else {
            i += 1;
        }
    }
}

/// Check for dead code (defined but never called procs).
fn check_dead_code(ctx: &ValidationContext, result: &mut ValidationResult) {
    for proc_name in &ctx.defined_procs {
        if !ctx.called_procs.contains(proc_name) {
            result.add_warning(
                format!("proc \"{proc_name}\" is defined but never called"),
                0,
            );
        }
    }
}

/// Find a similar name for "did you mean?" suggestions.
fn find_similar(name: &str, known: &HashSet<String>) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    for candidate in known {
        let dist = edit_distance(name, candidate);
        if dist <= 2 && best.as_ref().is_none_or(|(d, _)| dist < *d) {
            best = Some((dist, candidate.clone()));
        }
    }
    best.map(|(_, s)| format!("did you mean \"{s}\"?"))
}

/// Simple edit distance (Levenshtein).
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(n + 1) {
        *val = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;

    #[test]
    fn catches_unknown_command() {
        let interp = Interpreter::new();
        let result = interp.validate("putz hello");
        assert!(result.errors.iter().any(|d| d.message.contains("putz")));
    }

    #[test]
    fn suggests_similar_command() {
        let interp = Interpreter::new();
        let result = interp.validate("putz hello");
        let diag = result.errors.iter().find(|d| d.message.contains("putz"));
        assert!(diag.is_some());
        let suggestion = &diag.unwrap().suggestion;
        assert!(suggestion.as_ref().map_or(false, |s| s.contains("puts")));
    }

    #[test]
    fn valid_script_no_errors() {
        let interp = Interpreter::new();
        let result = interp.validate("set x 42\nputs $x");
        assert!(result.is_ok());
    }

    #[test]
    fn unreachable_code_warning() {
        let interp = Interpreter::new();
        let result = interp.validate("return 1\nputs hello");
        assert!(result
            .warnings
            .iter()
            .any(|d| d.message.contains("unreachable")));
    }

    #[test]
    fn undefined_variable_warning() {
        let interp = Interpreter::new();
        let result = interp.validate("puts $undefined_var");
        assert!(result
            .warnings
            .iter()
            .any(|d| d.message.contains("undefined_var")));
    }

    #[test]
    fn shadowing_warning() {
        let interp = Interpreter::new();
        let result = interp.validate("set x 1\nproc foo {} { set x 2 }");
        // x is set in outer scope, then set again in proc
        assert!(result
            .warnings
            .iter()
            .any(|d| d.message.contains("shadows")));
    }

    #[test]
    fn dead_code_warning() {
        let interp = Interpreter::new();
        let result = interp.validate("proc unused {} { return 1 }");
        assert!(result
            .warnings
            .iter()
            .any(|d| d.message.contains("never called")));
    }

    #[test]
    fn non_exhaustive_switch_warning() {
        let interp = Interpreter::new();
        let result = interp.validate("switch $x {a {puts a} b {puts b}}");
        assert!(result
            .warnings
            .iter()
            .any(|d| d.message.contains("exhaustive")));
    }

    #[test]
    fn edit_distance_works() {
        use super::edit_distance;
        assert_eq!(edit_distance("puts", "putz"), 1);
        assert_eq!(edit_distance("hello", "hello"), 0);
        assert_eq!(edit_distance("abc", "xyz"), 3);
    }
}
