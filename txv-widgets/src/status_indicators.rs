//! Status bar indicator items: ModeItem, PositionItem, BranchItem.
//! These are passive display items that react to commands from the editor.

use std::path::PathBuf;
use std::time::Instant;

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

/// Cursor position data emitted with CM_CURSOR_MOVED.
#[derive(Debug, Clone, Copy)]
pub struct CursorPos {
    pub line: u32,
    pub col: u32,
}


// --- ModeItem ---

/// Displays the current editor mode (NOR, INS, VIS, CMD).
pub struct ModeItem {
    command_id: CommandId,
    label_text: String,
}

impl ModeItem {
    pub fn new(command_id: CommandId) -> Self {
        Self { command_id, label_text: "NOR".to_string() }
    }
}

impl ActiveItem for ModeItem {
    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == self.command_id {
                if let Some(boxed) = data.as_ref() {
                    if let Some(mode) = boxed.downcast_ref::<String>() {
                        self.label_text = mode.clone();
                        return HandleResult::Consumed;
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for ModeItem {
    fn label(&self) -> &str { &self.label_text }
    fn gravity(&self) -> Gravity { Gravity::Right }
}

// --- PositionItem ---

/// Displays cursor position as "Ln N, Col M".
pub struct PositionItem {
    command_id: CommandId,
    label_text: String,
}

impl PositionItem {
    pub fn new(command_id: CommandId) -> Self {
        Self { command_id, label_text: "Ln 1, Col 1".to_string() }
    }
}

impl ActiveItem for PositionItem {
    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == self.command_id {
                if let Some(boxed) = data.as_ref() {
                    if let Some(pos) = boxed.downcast_ref::<CursorPos>() {
                        self.label_text = format!("Ln {}, Col {}", pos.line, pos.col);
                        return HandleResult::Consumed;
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for PositionItem {
    fn label(&self) -> &str { &self.label_text }
    fn gravity(&self) -> Gravity { Gravity::Right }
}

// --- BranchItem ---

/// Displays the current git branch name read from .git/HEAD.
pub struct BranchItem {
    root_dir: PathBuf,
    label_text: String,
    last_check: Instant,
}

impl BranchItem {
    pub fn new(root_dir: PathBuf) -> Self {
        let mut item = Self {
            root_dir,
            label_text: String::new(),
            last_check: Instant::now(),
        };
        item.refresh();
        item
    }

    fn refresh(&mut self) {
        self.label_text = Self::read_branch(&self.root_dir).unwrap_or_default();
        self.last_check = Instant::now();
    }

    fn read_branch(root: &PathBuf) -> Option<String> {
        let head = std::fs::read_to_string(root.join(".git/HEAD")).ok()?;
        let head = head.trim();
        if let Some(r) = head.strip_prefix("ref: refs/heads/") {
            Some(format!("\u{e0a0} {r}"))
        } else if head.len() >= 7 {
            Some(format!("\u{e0a0} {}", &head[..7]))
        } else {
            None
        }
    }
}

impl VisibleItem for BranchItem {
    fn label(&self) -> &str { &self.label_text }
    fn gravity(&self) -> Gravity { Gravity::Right }
    fn tick(&mut self) {
        if self.last_check.elapsed().as_secs() >= 30 {
            self.refresh();
        }
    }
}
