//! Error types for the rusticle interpreter.

use std::fmt;

/// Error codes that control interpreter flow.
#[derive(Clone, Debug, PartialEq)]
pub enum ErrorCode {
    /// A regular error.
    Error,
    /// A return statement with a value.
    Return(super::value::TclValue),
    /// A break statement.
    Break,
    /// A continue statement.
    Continue,
}

/// Source location for error reporting.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Location {
    /// Source file or description.
    pub source: String,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based).
    pub col: usize,
}

/// The main error type for the interpreter.
#[derive(Clone, Debug)]
pub struct TclError {
    /// Human-readable error message.
    pub message: String,
    /// Error code controlling flow.
    pub code: ErrorCode,
    /// Optional source location.
    pub location: Option<Location>,
}

impl TclError {
    /// Create a new error with the given message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: ErrorCode::Error,
            location: None,
        }
    }

    /// Create a new error with a specific code.
    pub fn with_code(message: impl Into<String>, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            code,
            location: None,
        }
    }

    /// Attach a source location to this error.
    pub fn at(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }
}

impl fmt::Display for TclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(loc) = &self.location {
            write!(
                f,
                "{}:{}:{}: {}",
                loc.source, loc.line, loc.col, self.message
            )
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for TclError {}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.source, self.line, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_no_location() {
        let err = TclError::new("something broke");
        assert_eq!(err.to_string(), "something broke");
    }

    #[test]
    fn error_display_with_location() {
        let err = TclError::new("bad command").at(Location {
            source: "test.tcl".into(),
            line: 5,
            col: 3,
        });
        assert_eq!(err.to_string(), "test.tcl:5:3: bad command");
    }

    #[test]
    fn error_code_default_is_error() {
        let err = TclError::new("oops");
        assert_eq!(err.code, ErrorCode::Error);
    }

    #[test]
    fn error_with_code() {
        let err = TclError::with_code("break", ErrorCode::Break);
        assert_eq!(err.code, ErrorCode::Break);
    }

    #[test]
    fn error_with_return_code() {
        let val = super::super::value::TclValue::Str("hello".into());
        let err = TclError::with_code("return", ErrorCode::Return(val.clone()));
        assert_eq!(err.code, ErrorCode::Return(val));
    }

    #[test]
    fn location_display() {
        let loc = Location {
            source: "script.tcl".into(),
            line: 10,
            col: 4,
        };
        assert_eq!(loc.to_string(), "script.tcl:10:4");
    }
}
