//! The rusticle interpreter: eval, scope chain, command dispatch.

pub(crate) mod eval;
mod scope;

use std::collections::HashMap;
use std::rc::Rc;

use crate::error::TclError;
use crate::value::TclValue;

/// Trait for commands that can be registered with the interpreter.
pub trait TclCommand: Send {
    /// Execute this command with the given arguments.
    fn call(&self, interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError>;
}

/// Wrapper to allow closures as commands.
struct FnCommand<F>(F);

impl<F> TclCommand for FnCommand<F>
where
    F: Fn(&mut Interpreter, &[TclValue]) -> Result<TclValue, TclError> + Send,
{
    fn call(&self, interp: &mut Interpreter, args: &[TclValue]) -> Result<TclValue, TclError> {
        (self.0)(interp, args)
    }
}

/// A procedure defined with `proc`.
#[derive(Clone, Debug)]
pub struct Proc {
    /// Parameter names.
    pub params: Vec<ProcParam>,
    /// The body script.
    pub body: String,
    /// The scope index where this proc was defined (for lexical scoping).
    pub defining_scope: usize,
}

/// A procedure parameter.
#[derive(Clone, Debug)]
pub struct ProcParam {
    /// Parameter name.
    pub name: String,
    /// Default value, if any.
    pub default: Option<TclValue>,
}

/// The interpreter.
pub struct Interpreter {
    /// The scope chain (stack of scopes).
    pub(crate) scopes: Vec<scope::Scope>,
    /// Registered commands.
    pub(crate) commands: HashMap<String, Rc<dyn TclCommand>>,
    /// Defined procedures.
    pub(crate) procs: HashMap<String, Proc>,
    /// Output captured by `puts`.
    pub(crate) output: Vec<String>,
    /// Named contexts.
    pub(crate) contexts: HashMap<String, crate::context::Context>,
    /// Currently executing context (for `declare`).
    pub(crate) current_context: Option<String>,
    /// Loaded command manifests.
    pub(crate) manifests: Vec<crate::manifest::Manifest>,
}

impl Interpreter {
    /// Create a new interpreter with built-in commands.
    pub fn new() -> Self {
        let mut interp = Self {
            scopes: vec![scope::Scope::new()],
            commands: HashMap::new(),
            procs: HashMap::new(),
            output: Vec::new(),
            contexts: HashMap::new(),
            current_context: None,
            manifests: Vec::new(),
        };
        crate::builtins::register_all(&mut interp);
        crate::context::register(&mut interp);
        crate::manifest::register(&mut interp);
        interp
    }

    /// Evaluate a script string.
    pub fn eval(&mut self, script: &str) -> Result<TclValue, TclError> {
        eval::eval_script_catching_return(self, script)
    }

    /// Evaluate a script with a source name for error reporting.
    pub fn eval_source(&mut self, script: &str, _source: &str) -> Result<TclValue, TclError> {
        self.eval(script)
    }

    /// Set a variable in the current scope.
    /// Validates type declarations for context variables.
    pub fn set_var(&mut self, name: &str, value: TclValue) {
        // Check context type declarations
        if name.contains("::") {
            if let Err(e) = crate::context::check_context_assignment(self, name, &value) {
                // Store the error message — caller should use try_set_var for Result
                self.output.push(format!("type error: {e}\n"));
                return;
            }
        }
        scope::set_var(&mut self.scopes, name, value);
    }

    /// Set a variable with type validation, returning errors.
    pub fn try_set_var(&mut self, name: &str, value: TclValue) -> Result<(), TclError> {
        if name.contains("::") {
            crate::context::check_context_assignment(self, name, &value)?;
        }
        scope::set_var(&mut self.scopes, name, value);
        Ok(())
    }

    /// Get a variable, walking up the scope chain.
    pub fn get_var(&self, name: &str) -> Option<&TclValue> {
        scope::get_var(&self.scopes, name)
    }

    /// Remove a variable from the current scope.
    pub fn unset_var(&mut self, name: &str) -> bool {
        scope::unset_var(&mut self.scopes, name)
    }

    /// Register an external command.
    pub fn register_command(&mut self, name: &str, cmd: Box<dyn TclCommand>) {
        self.commands.insert(name.to_string(), Rc::from(cmd));
    }

    /// Register a command from a closure.
    pub fn register_fn<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&mut Interpreter, &[TclValue]) -> Result<TclValue, TclError> + Send + 'static,
    {
        self.commands
            .insert(name.to_string(), Rc::new(FnCommand(f)));
    }

    /// Push a new scope onto the scope chain.
    pub fn push_scope(&mut self) {
        self.scopes.push(scope::Scope::new());
    }

    /// Push a new scope linked to a specific parent (for lexical scoping).
    pub(crate) fn push_scope_linked(&mut self, parent: usize) {
        self.scopes.push(scope::Scope::with_parent(parent));
    }

    /// Pop the current scope.
    pub(crate) fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Get captured output (from `puts`).
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    /// Clear captured output.
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Define a procedure.
    pub fn define_proc(&mut self, name: String, proc_def: Proc) {
        self.procs.insert(name, proc_def);
    }

    /// Set a variable in the parent scope (for `outer`).
    pub fn set_var_in_parent(
        &mut self,
        name: &str,
        value: TclValue,
        levels: usize,
    ) -> Result<(), TclError> {
        scope::set_var_in_parent(&mut self.scopes, name, value, levels)
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
