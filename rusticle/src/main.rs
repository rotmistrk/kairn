#![deny(clippy::unwrap_used, clippy::expect_used)]

//! rusticle — interactive REPL and script runner.

use std::io::{self, BufRead, Write};

use rusticle::error::ErrorCode;
use rusticle::interpreter::Interpreter;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        run_script(&args[1]);
    } else {
        run_repl();
    }
}

/// Run a script file.
fn run_script(path: &str) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            std::process::exit(1);
        }
    };
    let mut interp = Interpreter::new();
    match interp.eval_source(&content, path) {
        Ok(_) => print_output(&interp),
        Err(e) => {
            print_output(&interp);
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

/// Run the interactive REPL.
fn run_repl() {
    println!("rusticle — Tcl interpreter");
    println!("Type 'exit' to quit.\n");

    let mut interp = Interpreter::new();
    let stdin = io::stdin();
    let mut history: Vec<String> = Vec::new();

    loop {
        print!("% ");
        if io::stdout().flush().is_err() {
            break;
        }
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }
        history.push(trimmed.to_string());
        interp.clear_output();
        match interp.eval(trimmed) {
            Ok(val) => {
                print_output(&interp);
                let s = val.as_str();
                if !s.is_empty() {
                    println!("{s}");
                }
            }
            Err(e) => {
                print_output(&interp);
                match &e.code {
                    ErrorCode::Error => eprintln!("error: {}", e.message),
                    ErrorCode::Return(v) => println!("{}", v.as_str()),
                    ErrorCode::Break => eprintln!("error: break outside loop"),
                    ErrorCode::Continue => {
                        eprintln!("error: continue outside loop");
                    }
                }
            }
        }
    }
}

/// Print any captured output from the interpreter.
fn print_output(interp: &Interpreter) {
    for line in interp.get_output() {
        print!("{line}");
    }
}
