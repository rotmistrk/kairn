//! Load-time validation pass: static analysis for scripts.

use std::collections::{HashMap, HashSet};

use crate::error::Location;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::validate_checks::{check_dead_code, validate_commands};

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

    pub(crate) fn add_error(&mut self, msg: String, line: usize, suggestion: Option<String>) {
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

    pub(crate) fn add_warning(&mut self, msg: String, line: usize) {
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
pub(crate) struct ValidationContext {
    /// Known command names (builtins + procs).
    pub(crate) known_commands: HashSet<String>,
    /// Known variable names per scope.
    pub(crate) known_vars: Vec<HashSet<String>>,
    /// Defined procs with their arity (min, max).
    pub(crate) proc_arities: HashMap<String, (usize, usize)>,
    /// Called procs (for dead code detection).
    pub(crate) called_procs: HashSet<String>,
    /// Defined procs (for dead code detection).
    pub(crate) defined_procs: HashSet<String>,
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
            let min = proc_def.params.iter().filter(|p| p.default.is_none()).count();
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

    pub(crate) fn push_scope(&mut self) {
        self.known_vars.push(HashSet::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        if self.known_vars.len() > 1 {
            self.known_vars.pop();
        }
    }

    pub(crate) fn define_var(&mut self, name: &str) {
        if let Some(scope) = self.known_vars.last_mut() {
            scope.insert(name.to_string());
        }
    }

    pub(crate) fn is_var_known(&self, name: &str) -> bool {
        self.known_vars.iter().any(|s| s.contains(name))
    }

    pub(crate) fn is_var_in_current_scope(&self, name: &str) -> bool {
        self.known_vars.last().map(|s| s.contains(name)).unwrap_or(false)
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
