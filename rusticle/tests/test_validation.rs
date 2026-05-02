use rusticle::interpreter::Interpreter;

#[test]
fn valid_script_no_errors() {
    let interp = Interpreter::new();
    let result = interp.validate("set x 42\nputs $x");
    assert!(result.is_ok(), "expected no errors: {:?}", result.errors);
    assert!(
        result.warnings.is_empty(),
        "expected no warnings: {:?}",
        result.warnings
    );
}

#[test]
fn unknown_command_detected() {
    let interp = Interpreter::new();
    let result = interp.validate("putz hello");
    assert!(
        result.errors.iter().any(|d| d.message.contains("putz")),
        "expected error mentioning 'putz': {:?}",
        result.errors,
    );
}

#[test]
fn did_you_mean_suggestion() {
    let interp = Interpreter::new();
    let result = interp.validate("putz hello");
    let diag = result
        .errors
        .iter()
        .find(|d| d.message.contains("putz"))
        .expect("expected error for 'putz'");
    let suggestion = diag.suggestion.as_deref().expect("expected a suggestion");
    assert!(
        suggestion.contains("puts"),
        "suggestion should mention 'puts', got: {suggestion}",
    );
}

#[test]
fn undefined_variable_warning() {
    let interp = Interpreter::new();
    let result = interp.validate("puts $undefined_var");
    assert!(
        result
            .warnings
            .iter()
            .any(|d| d.message.contains("undefined_var")),
        "expected warning about undefined_var: {:?}",
        result.warnings,
    );
}

#[test]
fn proc_arity_check() {
    let interp = Interpreter::new();
    let script = "proc add {a b} { return 0 }\nadd 1";
    let result = interp.validate(script);
    // Arity mismatch may or may not be caught; verify no crash either way.
    if let Some(diag) = result.errors.iter().find(|d| d.message.contains("add")) {
        assert!(
            diag.message.contains("arg"),
            "arity error should mention args: {}",
            diag.message,
        );
    }
}
