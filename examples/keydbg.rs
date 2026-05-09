// Quick key debugger — run with: cargo run --example keydbg
use crossterm::{event, terminal};
use std::io::Write;

fn main() {
    terminal::enable_raw_mode().unwrap();
    println!("Press keys (Ctrl-C to quit):\r");
    loop {
        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            match event::read().unwrap() {
                event::Event::Key(k) => {
                    print!("  {:?}\r\n", k);
                    std::io::stdout().flush().unwrap();
                    if k.code == event::KeyCode::Char('c') && k.modifiers.contains(event::KeyModifiers::CONTROL) {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    terminal::disable_raw_mode().unwrap();
}
