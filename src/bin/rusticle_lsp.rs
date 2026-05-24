//! rusticle-lsp binary for integration tests.
//! Thin wrapper around rusticle::lsp.

use std::env;
use std::fs;
use std::io;

use rusticle::interpreter::Interpreter;
use rusticle::lsp::{read_message, write_message, Server};

fn main() {
    let mut prelude_paths: Vec<String> = Vec::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--prelude" {
            if let Some(path) = args.next() {
                prelude_paths.push(path);
            }
        }
    }

    let mut interp = Interpreter::new();
    for path in &prelude_paths {
        if let Ok(content) = fs::read_to_string(path) {
            let _ = interp.eval(&content);
        }
    }

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    let mut server = Server::new(interp);

    loop {
        let Some(request) = read_message(&mut reader) else {
            break;
        };
        let responses = server.handle(&request);
        for resp in responses {
            write_message(&mut writer, &resp);
        }
        if server.should_exit() {
            break;
        }
    }
}
