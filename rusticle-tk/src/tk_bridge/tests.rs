//! Unit tests for tk_bridge commands.

use txv_core::geometry::Rect;

use super::*;

fn setup() -> (Interpreter, Shared) {
    let mut interp = Interpreter::new();
    let shared = register_all(&mut interp);
    (interp, shared)
}

fn val(result: &Result<TclValue, rusticle::error::TclError>) -> String {
    match result {
        Ok(v) => v.as_str().into_owned(),
        Err(e) => panic!("eval failed: {e}"),
    }
}

#[test]
fn window_create_returns_id() {
    let (mut interp, _) = setup();
    let result = interp.eval(r#"window create "Test""#);
    assert_eq!(val(&result), "window_0");
}

#[test]
fn text_create_and_get() {
    let (mut interp, _) = setup();
    let _ = interp.eval(r#"set t [text create -content "hello"]"#);
    let result = interp.eval("text get $t");
    assert_eq!(val(&result), "hello");
}

#[test]
fn text_set_and_get() {
    let (mut interp, _) = setup();
    let _ = interp.eval("set t [text create]");
    let _ = interp.eval(r#"text set $t "world""#);
    let result = interp.eval("text get $t");
    assert_eq!(val(&result), "world");
}

#[test]
fn text_clear() {
    let (mut interp, _) = setup();
    let _ = interp.eval(r#"set t [text create -content "data"]"#);
    let _ = interp.eval("text clear $t");
    let result = interp.eval("text get $t");
    assert_eq!(val(&result), "");
}

#[test]
fn list_create_and_index() {
    let (mut interp, _) = setup();
    let _ = interp.eval("set l [list create]");
    let result = interp.eval("list index $l");
    assert_eq!(val(&result), "0");
}

#[test]
fn statusbar_create_and_set() {
    let (mut interp, _) = setup();
    let _ = interp.eval("set s [statusbar create]");
    let result = interp.eval(r#"statusbar left $s "Ready""#);
    assert!(result.is_ok());
}

#[test]
fn tabbar_create_add_active() {
    let (mut interp, _) = setup();
    let _ = interp.eval("set tb [tabbar create]");
    let _ = interp.eval(r#"tabbar add $tb "Tab1""#);
    let _ = interp.eval(r#"tabbar add $tb "Tab2""#);
    let result = interp.eval("tabbar active $tb");
    assert_eq!(val(&result), "0");
    let _ = interp.eval("tabbar set-active $tb 1");
    let result = interp.eval("tabbar active $tb");
    assert_eq!(val(&result), "1");
}

#[test]
fn progress_create_and_set() {
    let (mut interp, _) = setup();
    let _ = interp.eval("set p [progress create]");
    let result = interp.eval("progress set $p 0.5");
    assert!(result.is_ok());
    let result = interp.eval("progress done $p");
    assert!(result.is_ok());
}

#[test]
fn bind_registers_key() {
    let (mut interp, shared) = setup();
    let _ = interp.eval(r#"bind Ctrl-Q { app quit }"#);
    let st = shared.lock().unwrap_or_else(|e| e.into_inner());
    assert!(st.events.key_bindings.contains_key("Ctrl-Q"));
}

#[test]
fn app_run_sets_flag() {
    let (mut interp, shared) = setup();
    let _ = interp.eval("app run");
    let st = shared.lock().unwrap_or_else(|e| e.into_inner());
    assert!(st.run_requested);
}

#[test]
fn app_quit_sets_flag() {
    let (mut interp, shared) = setup();
    let _ = interp.eval("app quit");
    let st = shared.lock().unwrap_or_else(|e| e.into_inner());
    assert!(st.events.quit_requested);
}

#[test]
fn dialog_confirm_returns_bool() {
    let (mut interp, _) = setup();
    let result = interp.eval(r#"dialog confirm "Sure?""#);
    assert_eq!(val(&result), "0");
}

#[test]
fn input_create_get_set() {
    let (mut interp, _) = setup();
    let _ = interp.eval("set i [input create]");
    let _ = interp.eval(r#"input set $i "hello""#);
    let result = interp.eval("input get $i");
    assert_eq!(val(&result), "hello");
}

#[test]
fn input_clear() {
    let (mut interp, _) = setup();
    let _ = interp.eval("set i [input create]");
    let _ = interp.eval(r#"input set $i "data""#);
    let _ = interp.eval("input clear $i");
    let result = interp.eval("input get $i");
    assert_eq!(val(&result), "");
}

#[test]
fn after_registers_timer() {
    let (mut interp, shared) = setup();
    let _ = interp.eval(r#"after 1000 { puts "tick" }"#);
    let st = shared.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(st.events.timers.len(), 1);
    assert!(!st.events.timers[0].repeat);
}

#[test]
fn after_repeat_registers_timer() {
    let (mut interp, shared) = setup();
    let _ = interp.eval(r#"after 1000 -repeat { puts "tick" }"#);
    let st = shared.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(st.events.timers.len(), 1);
    assert!(st.events.timers[0].repeat);
}

#[test]
fn hello_script_parses() {
    let (mut interp, shared) = setup();
    let script = r#"
        set win [window create "Hello"]
        set txt [text create -content "Hello, rusticle-tk!"]
        window add $win $txt -side fill
        set status [statusbar create]
        statusbar left $status "Ready"
        window add $win $status -side bottom -height 1
        bind Ctrl-Q { app quit }
        app run
    "#;
    let result = interp.eval(script);
    assert!(result.is_ok(), "hello script failed: {result:?}");
    let st = shared.lock().unwrap_or_else(|e| e.into_inner());
    assert!(st.run_requested);
    assert!(st.events.key_bindings.contains_key("Ctrl-Q"));
}

#[test]
fn dialog_demo_script_parses() {
    let (mut interp, _) = setup();
    let script = r#"
        set answer [dialog confirm "Proceed?"]
        if {$answer} { puts "yes" } else { puts "no" }
    "#;
    let result = interp.eval(script);
    assert!(result.is_ok(), "dialog demo failed: {result:?}");
    let output = interp.get_output().join("");
    assert!(output.contains("no"));
}

#[test]
fn window_add_updates_layout() {
    let (mut interp, shared) = setup();
    let _ = interp.eval(r#"set w [window create "T"]"#);
    let _ = interp.eval("set t [text create]");
    let _ = interp.eval("window add $w $t -side fill");
    let st = shared.lock().unwrap_or_else(|e| e.into_inner());
    let rects = st.desktop.layout.compute(Rect::new(0, 0, 80, 24));
    assert_eq!(rects.len(), 1);
}

#[test]
fn table_create() {
    let (mut interp, _) = setup();
    let _ = interp.eval(r#"set t [table create -columns "Name Size"]"#);
    let result = interp.eval("table selected $t");
    assert_eq!(val(&result), "0");
}
