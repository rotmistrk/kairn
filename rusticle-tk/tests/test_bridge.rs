use rusticle::interpreter::Interpreter;
use rusticle_tk::tk_bridge;

fn setup() -> Interpreter {
    let mut interp = Interpreter::new();
    let _ = tk_bridge::register_all(&mut interp);
    interp
}

fn eval_ok(interp: &mut Interpreter, script: &str) -> String {
    let result = interp.eval(script);
    result.unwrap().as_str().into_owned()
}

// ── text widget ─────────────────────────────────────────────────────

#[test]
fn text_create_returns_id() {
    let mut interp = setup();
    let id = eval_ok(&mut interp, "text create");
    assert!(
        id.starts_with("widget_"),
        "expected widget_ prefix, got: {id}"
    );
}

#[test]
fn text_set_get_roundtrip() {
    let mut interp = setup();
    eval_ok(&mut interp, "set t [text create]");
    eval_ok(&mut interp, r#"text set $t "hello world""#);
    let val = eval_ok(&mut interp, "text get $t");
    assert_eq!(val, "hello world");
}

#[test]
fn text_create_with_content() {
    let mut interp = setup();
    eval_ok(&mut interp, r#"set t [text create -content "hello"]"#);
    let val = eval_ok(&mut interp, "text get $t");
    assert_eq!(val, "hello");
}

#[test]
fn text_clear_empties() {
    let mut interp = setup();
    eval_ok(&mut interp, r#"set t [text create -content "data"]"#);
    eval_ok(&mut interp, "text clear $t");
    let val = eval_ok(&mut interp, "text get $t");
    assert_eq!(val, "");
}

// ── list widget ─────────────────────────────────────────────────────

#[test]
fn list_create_returns_id() {
    let mut interp = setup();
    let id = eval_ok(&mut interp, "list create");
    assert!(
        id.starts_with("widget_"),
        "expected widget_ prefix, got: {id}"
    );
}

#[test]
fn list_selected_default() {
    let mut interp = setup();
    eval_ok(&mut interp, "set l [list create]");
    let val = eval_ok(&mut interp, "list selected $l");
    assert_eq!(val, "");
}

// ── tree widget ─────────────────────────────────────────────────────

#[test]
fn tree_create_returns_id() {
    let mut interp = setup();
    let id = eval_ok(&mut interp, r#"tree create -data ".""#);
    assert!(
        id.starts_with("widget_"),
        "expected tree_ prefix, got: {id}"
    );
}

// ── input widget ────────────────────────────────────────────────────

#[test]
fn input_create_get_set() {
    let mut interp = setup();
    eval_ok(&mut interp, r#"set i [input create -prompt "> "]"#);
    eval_ok(&mut interp, r#"input set $i "typed""#);
    let val = eval_ok(&mut interp, "input get $i");
    assert_eq!(val, "typed");
}

#[test]
fn input_clear_empties() {
    let mut interp = setup();
    eval_ok(&mut interp, "set i [input create]");
    eval_ok(&mut interp, r#"input set $i "stuff""#);
    eval_ok(&mut interp, "input clear $i");
    let val = eval_ok(&mut interp, "input get $i");
    assert_eq!(val, "");
}

// ── statusbar widget ────────────────────────────────────────────────

#[test]
fn statusbar_create_and_left() {
    let mut interp = setup();
    eval_ok(&mut interp, "set s [statusbar create]");
    let result = interp.eval(r#"statusbar left $s "Ready""#);
    assert!(result.is_ok());
}

#[test]
fn statusbar_right() {
    let mut interp = setup();
    eval_ok(&mut interp, "set s [statusbar create]");
    let result = interp.eval(r#"statusbar right $s "Ln 42""#);
    assert!(result.is_ok());
}

// ── tabbar widget ───────────────────────────────────────────────────

#[test]
fn tabbar_create_add_active() {
    let mut interp = setup();
    eval_ok(&mut interp, "set tb [tabbar create]");
    eval_ok(&mut interp, r#"tabbar add $tb "Tab1""#);
    eval_ok(&mut interp, r#"tabbar add $tb "Tab2""#);
    let active = eval_ok(&mut interp, "tabbar active $tb");
    assert_eq!(active, "0");
    eval_ok(&mut interp, "tabbar set-active $tb 1");
    let active = eval_ok(&mut interp, "tabbar active $tb");
    assert_eq!(active, "1");
}

// ── table widget ────────────────────────────────────────────────────

#[test]
fn table_create_returns_id() {
    let mut interp = setup();
    let id = eval_ok(&mut interp, r#"table create -columns "A B""#);
    assert!(
        id.starts_with("widget_"),
        "expected widget_ prefix, got: {id}"
    );
}

#[test]
fn table_add_row_no_error() {
    let mut interp = setup();
    eval_ok(&mut interp, r#"set t [table create -columns "A B"]"#);
    let result = interp.eval(r#"table add-row $t "cell1 cell2""#);
    assert!(result.is_ok());
}

// ── progress widget ─────────────────────────────────────────────────

#[test]
fn progress_create_and_set() {
    let mut interp = setup();
    eval_ok(&mut interp, r#"set p [progress create -title "Build"]"#);
    let result = interp.eval("progress set $p 0.5");
    assert!(result.is_ok());
}

#[test]
fn progress_done() {
    let mut interp = setup();
    eval_ok(&mut interp, "set p [progress create]");
    let result = interp.eval("progress done $p");
    assert!(result.is_ok());
}

// ── dialog commands ─────────────────────────────────────────────────

#[test]
fn dialog_confirm_returns_false() {
    let mut interp = setup();
    let val = eval_ok(&mut interp, r#"dialog confirm "Sure?""#);
    assert_eq!(val, "0");
}

#[test]
fn dialog_info_no_error() {
    let mut interp = setup();
    let result = interp.eval(r#"dialog info "Hello""#);
    assert!(result.is_ok());
}

#[test]
fn dialog_prompt_returns_default() {
    let mut interp = setup();
    let val = eval_ok(&mut interp, r#"dialog prompt "Name:" "default""#);
    assert_eq!(val, "default");
}

// ── notify ──────────────────────────────────────────────────────────

#[test]
fn notify_no_error() {
    let mut interp = setup();
    let result = interp.eval(r#"notify "Hello""#);
    assert!(result.is_ok());
}

// ── bind ────────────────────────────────────────────────────────────

#[test]
fn bind_no_error() {
    let mut interp = setup();
    let result = interp.eval(r#"bind Ctrl-Q { app quit }"#);
    assert!(result.is_ok());
}

// ── after (timers) ──────────────────────────────────────────────────

#[test]
fn after_no_error() {
    let mut interp = setup();
    let result = interp.eval(r#"after 1000 { puts tick }"#);
    assert!(result.is_ok());
}

#[test]
fn after_repeat_no_error() {
    let mut interp = setup();
    let result = interp.eval(r#"after 1000 -repeat { puts tick }"#);
    assert!(result.is_ok());
}

// ── window commands ─────────────────────────────────────────────────

#[test]
fn window_create_returns_id() {
    let mut interp = setup();
    let val = eval_ok(&mut interp, r#"window create "Test""#);
    assert_eq!(val, "window_0");
}

#[test]
fn window_add_no_error() {
    let mut interp = setup();
    eval_ok(&mut interp, r#"set w [window create "App"]"#);
    eval_ok(&mut interp, "set t [text create]");
    let result = interp.eval("window add $w $t -side fill");
    assert!(result.is_ok());
}
