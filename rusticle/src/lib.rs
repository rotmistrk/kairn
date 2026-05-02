#![deny(clippy::unwrap_used, clippy::expect_used)]

//! rusticle — a Tcl-compatible scripting language interpreter with modern enhancements.
//!
//! Features: lexical scoping, typed declarations, structured literals,
//! accessor syntax, pipes, pattern matching, and load-time static analysis.

pub mod builtins;
pub mod context;
pub mod error;
pub mod interpreter;
pub mod manifest;
pub mod parser;
pub mod types;
pub mod validate;
pub mod value;
