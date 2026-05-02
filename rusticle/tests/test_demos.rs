use rusticle::interpreter::Interpreter;
use std::fs;

#[test]
fn demo_basics() {
    let script = fs::read_to_string("examples/basics.tcl").unwrap();
    let mut interp = Interpreter::new();
    interp.eval(&script).unwrap();
    let output = interp.get_output().join("");
    assert!(output.contains("=== Done ==="), "output: {output}");
}

#[test]
fn demo_data_structures() {
    let script = fs::read_to_string("examples/data_structures.tcl").unwrap();
    let mut interp = Interpreter::new();
    interp.eval(&script).unwrap();
    let output = interp.get_output().join("");
    assert!(output.contains("=== Done ==="), "output: {output}");
}

#[test]
fn demo_functional() {
    let script = fs::read_to_string("examples/functional.tcl").unwrap();
    let mut interp = Interpreter::new();
    interp.eval(&script).unwrap();
    let output = interp.get_output().join("");
    assert!(output.contains("=== Done ==="), "output: {output}");
}

#[test]
fn demo_contexts() {
    let script = fs::read_to_string("examples/contexts.tcl").unwrap();
    let mut interp = Interpreter::new();
    interp.eval(&script).unwrap();
    let output = interp.get_output().join("");
    assert!(output.contains("=== Done ==="), "output: {output}");
}
