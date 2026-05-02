//! Tcl command parser with syntactic enhancements.
//!
//! Handles: word splitting, variable/command substitution, quoting,
//! structured literals (%{} %[]), accessors, pipes, destructuring,
//! optional chaining, ranges, heredoc, and pattern matching.

mod core;
mod literals;
mod subst;
mod words;

pub(crate) use self::core::Word;
pub use self::core::{Command, ParsedScript, Parser};
