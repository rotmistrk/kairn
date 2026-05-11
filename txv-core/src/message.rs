//! Message — structured application message with severity and origin.

use std::time::Instant;

/// Severity level for messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl MsgLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Debug => "DBG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERR",
        }
    }
}

/// A single application message.
#[derive(Clone)]
pub struct Message {
    pub level: MsgLevel,
    pub origin: &'static str,
    pub text: String,
    pub timestamp: Instant,
}

impl Message {
    pub fn new(level: MsgLevel, origin: &'static str, text: impl Into<String>) -> Self {
        Self {
            level,
            origin,
            text: text.into(),
            timestamp: Instant::now(),
        }
    }

    pub fn info(origin: &'static str, text: impl Into<String>) -> Self {
        Self::new(MsgLevel::Info, origin, text)
    }

    pub fn warn(origin: &'static str, text: impl Into<String>) -> Self {
        Self::new(MsgLevel::Warn, origin, text)
    }

    pub fn error(origin: &'static str, text: impl Into<String>) -> Self {
        Self::new(MsgLevel::Error, origin, text)
    }
}
