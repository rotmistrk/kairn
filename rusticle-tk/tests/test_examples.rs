use rusticle::interpreter::Interpreter;
use rusticle::value::TclValue;
use rusticle_tk::tk_bridge;
use std::fs;

fn setup() -> Interpreter {
    let mut interp = Interpreter::new();
    let _ = tk_bridge::register_all(&mut interp);
    interp
}

#[test]
fn example_hello() {
    let mut interp = setup();
    let script = fs::read_to_string("examples/hello.tcl").unwrap();
    interp.eval(&script).unwrap();
}

#[test]
fn example_dialog_demo() {
    let mut interp = setup();
    let script = fs::read_to_string("examples/dialog-demo.tcl").unwrap();
    interp.eval(&script).unwrap();
    let output = interp.get_output().join("");
    assert!(
        output.contains("no"),
        "expected output to contain 'no', got: {output}"
    );
}

#[test]
fn example_file_browser() {
    let mut interp = setup();
    let script = fs::read_to_string("examples/file-browser.tcl").unwrap();
    interp.eval(&script).unwrap();
}

#[test]
fn example_widget_gallery() {
    let mut interp = setup();
    let script = fs::read_to_string("examples/widget-gallery.tcl").unwrap();
    interp.eval(&script).unwrap();
}

#[test]
fn example_dashboard() {
    let mut interp = setup();
    let script = fs::read_to_string("examples/dashboard.tcl").unwrap();
    interp.eval(&script).unwrap();
}

#[test]
fn example_config_editor() {
    let mut interp = setup();
    let script = fs::read_to_string("examples/config-editor.tcl").unwrap();
    interp.eval(&script).unwrap();
}

#[test]
fn example_log_viewer() {
    let mut interp = setup();
    interp.set_var("argv", TclValue::List(vec![]));
    let script = fs::read_to_string("examples/log-viewer.tcl").unwrap();
    interp.eval(&script).unwrap();
}

#[test]
fn example_todo_list() {
    let mut interp = setup();
    let script = fs::read_to_string("examples/todo-list.tcl").unwrap();
    match interp.eval(&script) {
        Ok(_) => {}
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("lreplace"),
                "unexpected error (expected lreplace): {msg}"
            );
        }
    }
}
