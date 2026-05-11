use super::*;
use txv_core::cell::Color;

#[test]
fn basic_print() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"Hello");
    assert_eq!(tb.cells[0][0].ch, 'H');
    assert_eq!(tb.cells[0][4].ch, 'o');
    assert_eq!(tb.cursor(), (5, 0));
}

#[test]
fn newline_and_cr() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"A\r\nB");
    assert_eq!(tb.cells[0][0].ch, 'A');
    assert_eq!(tb.cells[1][0].ch, 'B');
}

#[test]
fn cursor_movement() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"\x1b[5;10H");
    assert_eq!(tb.cursor(), (9, 4));
}

#[test]
fn erase_line() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"ABCDEF\x1b[4G\x1b[K");
    assert_eq!(tb.cells[0][0].ch, 'A');
    assert_eq!(tb.cells[0][1].ch, 'B');
    assert_eq!(tb.cells[0][2].ch, 'C');
    assert_eq!(tb.cells[0][3].ch, ' ');
}

#[test]
fn sgr_colors() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"\x1b[31mR\x1b[0m");
    assert_eq!(tb.cells[0][0].ch, 'R');
    assert_eq!(tb.cells[0][0].style.fg, Color::Ansi(1));
}

#[test]
fn scroll_on_overflow() {
    let mut tb = TermBuf::new(80, 3);
    tb.process(b"A\r\nB\r\nC\r\nD");
    assert_eq!(tb.cells[0][0].ch, 'B');
    assert_eq!(tb.cells[1][0].ch, 'C');
    assert_eq!(tb.cells[2][0].ch, 'D');
}

#[test]
fn render_to_surface() {
    let mut tb = TermBuf::new(10, 5);
    tb.process(b"Hi");
    let mut surface = Surface::new(10, 5);
    tb.render_to(&mut surface);
    assert_eq!(surface.cell(0, 0).ch, 'H');
    assert_eq!(surface.cell(1, 0).ch, 'i');
}

#[test]
fn resize_preserves_content() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"Hello");
    tb.resize(40, 12);
    assert_eq!(tb.cells[0][0].ch, 'H');
    assert_eq!(tb.cols, 40);
    assert_eq!(tb.rows, 12);
}

#[test]
fn cursor_visibility() {
    let mut tb = TermBuf::new(80, 24);
    assert!(tb.cursor_visible());
    tb.process(b"\x1b[?25l");
    assert!(!tb.cursor_visible());
    tb.process(b"\x1b[?25h");
    assert!(tb.cursor_visible());
}

#[test]
fn swallow_esc_k_title_sequence() {
    let mut tb = TermBuf::new(80, 24);
    // ESC k sets tmux title — content should NOT appear on screen
    tb.process(b"A\x1bktitle text\x1b\\B");
    assert_eq!(tb.cells[0][0].ch, 'A');
    assert_eq!(tb.cells[0][1].ch, 'B');
    assert_eq!(tb.cursor(), (2, 0));
}

#[test]
fn swallow_esc_k_terminated_by_bel() {
    let mut tb = TermBuf::new(80, 24);
    tb.process(b"X\x1bkmy title\x07Y");
    assert_eq!(tb.cells[0][0].ch, 'X');
    assert_eq!(tb.cells[0][1].ch, 'Y');
}

#[test]
fn swallow_real_prompt_esc_k() {
    let mut tb = TermBuf::new(80, 24);
    // Real prompt: ESC[32m ESC_k kairn/f4 ESC\ core ESC[34m : ...
    tb.process(b"\x1b[00;32m\x1bkkairn/f4\x1b\\core\x1b[01;34m:");
    // Should see "core:" without "kairn/f4"
    let row: String = tb.cells[0].iter().take(10).map(|c| c.ch).collect();
    assert_eq!(row.trim(), "core:");
}
