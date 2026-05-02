use rusticle::interpreter::Interpreter;

#[test]
fn list_creation() {
    let mut interp = Interpreter::new();
    let result = interp.eval("list a b c").unwrap();
    assert_eq!(result.as_str().as_ref(), "a b c");
}

#[test]
fn lindex_first() {
    let mut interp = Interpreter::new();
    let result = interp.eval("lindex [list a b c] 0").unwrap();
    assert_eq!(result.as_str().as_ref(), "a");
}

#[test]
fn lindex_last() {
    let mut interp = Interpreter::new();
    let result = interp.eval("lindex [list a b c] 2").unwrap();
    assert_eq!(result.as_str().as_ref(), "c");
}

#[test]
fn llength_basic() {
    let mut interp = Interpreter::new();
    let result = interp.eval("llength [list a b c]").unwrap();
    assert_eq!(result.as_int().unwrap(), 3);
}

#[test]
fn llength_empty() {
    let mut interp = Interpreter::new();
    let result = interp.eval("llength [list]").unwrap();
    assert_eq!(result.as_int().unwrap(), 0);
}

#[test]
fn lappend_to_list() {
    let mut interp = Interpreter::new();
    interp.eval("set x [list a b]").unwrap();
    interp.eval("lappend x c").unwrap();
    let result = interp.eval("set x").unwrap();
    assert_eq!(result.as_str().as_ref(), "a b c");
}

#[test]
fn lrange_subset() {
    let mut interp = Interpreter::new();
    let result = interp.eval("lrange [list a b c d e] 1 3").unwrap();
    assert_eq!(result.as_str().as_ref(), "b c d");
}

#[test]
fn lsearch_found() {
    let mut interp = Interpreter::new();
    let result = interp.eval("lsearch [list a b c] b").unwrap();
    assert_eq!(result.as_int().unwrap(), 1);
}

#[test]
fn lsearch_not_found() {
    let mut interp = Interpreter::new();
    let result = interp.eval("lsearch [list a b c] z").unwrap();
    assert_eq!(result.as_int().unwrap(), -1);
}

#[test]
fn lsort_alphabetical() {
    let mut interp = Interpreter::new();
    let result = interp.eval("lsort [list c a b]").unwrap();
    assert_eq!(result.as_str().as_ref(), "a b c");
}

#[test]
fn lset_modifies() {
    let mut interp = Interpreter::new();
    interp.eval("set x [list a b c]").unwrap();
    interp.eval("lset x 1 B").unwrap();
    let result = interp.eval("set x").unwrap();
    assert_eq!(result.as_str().as_ref(), "a B c");
}

#[test]
fn lmap_double() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("lmap [list 1 2 3] {x { expr {$x * 2} }}")
        .unwrap();
    assert_eq!(result.as_str().as_ref(), "2 4 6");
}

#[test]
fn lfilter_evens() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("lfilter [range 1 10] {x { expr {$x % 2 == 0} }}")
        .unwrap();
    assert_eq!(result.as_str().as_ref(), "2 4 6 8");
}

#[test]
fn lreduce_sum() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval("lreduce [list 1 2 3 4 5] 0 {acc x { expr {$acc + $x} }}")
        .unwrap();
    assert_eq!(result.as_int().unwrap(), 15);
}

#[test]
fn range_basic() {
    let mut interp = Interpreter::new();
    let result = interp.eval("range 1 5").unwrap();
    assert_eq!(result.as_str().as_ref(), "1 2 3 4");
}

#[test]
fn range_with_step() {
    let mut interp = Interpreter::new();
    let result = interp.eval("range 0 10 3").unwrap();
    assert_eq!(result.as_str().as_ref(), "0 3 6 9");
}

#[test]
fn empty_list_operations() {
    let mut interp = Interpreter::new();
    let len = interp.eval("llength [list]").unwrap();
    assert_eq!(len.as_int().unwrap(), 0);
    let elem = interp.eval("lindex [list] 0").unwrap();
    assert_eq!(elem.as_str().as_ref(), "");
}

#[test]
fn nested_lists() {
    let mut interp = Interpreter::new();
    interp.eval("set x [list [list 1 2] [list 3 4]]").unwrap();
    let result = interp.eval("llength $x").unwrap();
    assert_eq!(result.as_int().unwrap(), 2);
}
