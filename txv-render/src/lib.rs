//! # txv-render
//!
//! Terminal backend for the txv TUI framework.
//! Implements `txv_core::Backend` via crossterm, provides a VTE terminal
//! emulator (`TermBuf`), color mode detection/downgrade, and text utilities.

pub mod backend;
pub mod color;
mod event_translate;
pub mod termbuf;
pub mod text;

pub use backend::CrosstermBackend;
pub use color::{detect_color_mode, downgrade, ColorMode};
pub use termbuf::TermBuf;
pub use text::{byte_to_col, col_to_byte, display_width, truncate, wrap};
