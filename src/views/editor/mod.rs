//! EditorView — kairn's editor: type alias for txv_edit EditorView<KairnDelegate>.
//!
//! KairnDelegate provides:
//! - Git gutter signs
//! - Diagnostics underlines
//! - Blame gutter
//! - LSP integration (completion, signature, goto)
//! - Autosave, disk change detection
//!
//! Diff mode is a SEPARATE DiffView component, not part of EditorView.

pub mod build;
mod delegate;
mod delegate_accessors;
mod delegate_diff;
pub mod diff_model;
pub mod diff_opts;
mod handle_action;
mod handle_command_event;
mod handle_completion;
mod handle_completion_accept;
mod handle_deferred;
mod handle_signature;
mod handle_tick;
pub mod methods;
pub mod methods_diff;
pub mod sbs_model;

pub use delegate::KairnDelegate;
pub use methods::EditorViewExt;
pub use methods_diff::EditorViewDiffExt;

/// Kairn's editor view.
pub type EditorView = txv_edit::view::EditorView<KairnDelegate>;
