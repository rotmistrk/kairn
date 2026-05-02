use rusticle::interpreter::Interpreter;
use rusticle_tk::tk_bridge;

fn setup() -> (Interpreter, rusticle_tk::tk_bridge::Shared) {
    let mut interp = Interpreter::new();
    let shared = tk_bridge::register_all(&mut interp);
    (interp, shared)
}

fn area() -> txv::layout::Rect {
    txv::layout::Rect {
        x: 0,
        y: 0,
        w: 80,
        h: 24,
    }
}

fn eval(interp: &mut Interpreter, script: &str) {
    interp
        .eval(script)
        .unwrap_or_else(|e| panic!("eval failed: {e}"));
}

#[test]
fn single_fill_widget() {
    let (mut interp, shared) = setup();
    eval(&mut interp, r#"set w [window create "test"]"#);
    eval(&mut interp, "set t [text create]");
    eval(&mut interp, "window add $w $t -side fill");

    let st = shared.lock().unwrap();
    let rects = st.layout.compute(area());
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].1, area());
}

#[test]
fn left_and_fill() {
    let (mut interp, shared) = setup();
    eval(&mut interp, r#"set w [window create "test"]"#);
    eval(&mut interp, "set a [text create]");
    eval(&mut interp, "set b [text create]");
    eval(&mut interp, "window add $w $a -side left -width 20");
    eval(&mut interp, "window add $w $b -side fill");

    let st = shared.lock().unwrap();
    let rects = st.layout.compute(area());
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].1.w, 20);
    assert_eq!(rects[0].1.x, 0);
    assert_eq!(rects[1].1.w, 60);
    assert_eq!(rects[1].1.x, 20);
}

#[test]
fn bottom_and_fill() {
    let (mut interp, shared) = setup();
    eval(&mut interp, r#"set w [window create "test"]"#);
    eval(&mut interp, "set a [statusbar create]");
    eval(&mut interp, "set b [text create]");
    eval(&mut interp, "window add $w $a -side bottom -height 1");
    eval(&mut interp, "window add $w $b -side fill");

    let st = shared.lock().unwrap();
    let rects = st.layout.compute(area());
    assert_eq!(rects.len(), 2);
    assert_eq!(rects[0].1.h, 1);
    assert_eq!(rects[0].1.y, 23);
    assert_eq!(rects[1].1.h, 23);
    assert_eq!(rects[1].1.y, 0);
}

#[test]
fn three_panel_layout() {
    let (mut interp, shared) = setup();
    eval(&mut interp, r#"set w [window create "test"]"#);
    eval(&mut interp, "set tree [text create]");
    eval(&mut interp, "set status [statusbar create]");
    eval(&mut interp, "set main [text create]");
    eval(&mut interp, "window add $w $tree -side left -width 25");
    eval(&mut interp, "window add $w $status -side bottom -height 1");
    eval(&mut interp, "window add $w $main -side fill");

    let st = shared.lock().unwrap();
    let rects = st.layout.compute(area());
    assert_eq!(rects.len(), 3);
    // Left panel: w=25, full height
    assert_eq!(rects[0].1.w, 25);
    assert_eq!(rects[0].1.h, 24);
    // Bottom status: h=1, remaining width
    assert_eq!(rects[1].1.h, 1);
    assert_eq!(rects[1].1.w, 55);
    // Fill: remaining area
    assert_eq!(rects[2].1.w, 55);
    assert_eq!(rects[2].1.h, 23);
}

#[test]
fn multiple_widgets_mixed_sides() {
    let (mut interp, shared) = setup();
    eval(&mut interp, r#"set w [window create "test"]"#);
    eval(&mut interp, "set top [statusbar create]");
    eval(&mut interp, "set bot [statusbar create]");
    eval(&mut interp, "set left [text create]");
    eval(&mut interp, "set main [text create]");
    eval(&mut interp, "window add $w $top -side top -height 1");
    eval(&mut interp, "window add $w $bot -side bottom -height 1");
    eval(&mut interp, "window add $w $left -side left -width 20");
    eval(&mut interp, "window add $w $main -side fill");

    let st = shared.lock().unwrap();
    let rects = st.layout.compute(area());
    assert_eq!(rects.len(), 4);
    // Top: h=1, full width
    assert_eq!(rects[0].1.h, 1);
    assert_eq!(rects[0].1.w, 80);
    assert_eq!(rects[0].1.y, 0);
    // Bottom: h=1, full width, at y=23
    assert_eq!(rects[1].1.h, 1);
    assert_eq!(rects[1].1.w, 80);
    assert_eq!(rects[1].1.y, 23);
    // Left: w=20, remaining height (22)
    assert_eq!(rects[2].1.w, 20);
    assert_eq!(rects[2].1.h, 22);
    // Fill: remaining
    assert_eq!(rects[3].1.w, 60);
    assert_eq!(rects[3].1.h, 22);
}
