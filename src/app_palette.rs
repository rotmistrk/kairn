//! Application-level palette — extends txv-core Palette with domain-specific roles.

#[path = "app_palette_defaults.rs"]
mod defaults;
#[path = "app_palette_roles.rs"]
mod roles;
#[path = "app_palette_roles2.rs"]
mod roles2;

pub use roles::*;
pub use roles2::*;

use std::sync::{OnceLock, RwLock};

static APP_PALETTE: OnceLock<RwLock<AppPalette>> = OnceLock::new();

/// Get the active app palette.
pub fn app_palette() -> AppPalette {
    match APP_PALETTE.get() {
        Some(lock) => lock.read().map(|p| p.clone()).unwrap_or_default(),
        None => AppPalette::default(),
    }
}

/// Set the active app palette.
pub fn set_app_palette(p: &AppPalette) {
    match APP_PALETTE.get() {
        Some(lock) => {
            if let Ok(mut w) = lock.write() {
                *w = p.clone();
            }
        }
        None => {
            let _ = APP_PALETTE.set(RwLock::new(p.clone()));
        }
    }
}

/// kairn-specific palette extending the framework palette.
#[derive(Clone, Debug)]
pub struct AppPalette {
    pub(crate) git: GitPalette,
    pub(crate) diff: DiffPalette,
    pub(crate) editor: EditorPalette,
    pub(crate) diag: DiagPalette,
    pub(crate) tree: TreePalette,
    pub(crate) todo: TodoPalette,
    pub(crate) msg: MsgPalette,
    pub(crate) badge: BadgePalette,
    pub(crate) roots: RootsPalette,
}

impl AppPalette {
    pub fn git(&self) -> &GitPalette {
        &self.git
    }
    pub fn diff(&self) -> &DiffPalette {
        &self.diff
    }
    pub fn editor(&self) -> &EditorPalette {
        &self.editor
    }
    pub fn diag(&self) -> &DiagPalette {
        &self.diag
    }
    pub fn tree(&self) -> &TreePalette {
        &self.tree
    }
    pub fn todo(&self) -> &TodoPalette {
        &self.todo
    }
    pub fn msg(&self) -> &MsgPalette {
        &self.msg
    }
    pub fn badge(&self) -> &BadgePalette {
        &self.badge
    }
    pub fn roots(&self) -> &RootsPalette {
        &self.roots
    }

    pub fn git_mut(&mut self) -> &mut GitPalette {
        &mut self.git
    }
    pub fn diff_mut(&mut self) -> &mut DiffPalette {
        &mut self.diff
    }
    pub fn editor_mut(&mut self) -> &mut EditorPalette {
        &mut self.editor
    }
    pub fn diag_mut(&mut self) -> &mut DiagPalette {
        &mut self.diag
    }
    pub fn tree_mut(&mut self) -> &mut TreePalette {
        &mut self.tree
    }
    pub fn todo_mut(&mut self) -> &mut TodoPalette {
        &mut self.todo
    }
    pub fn msg_mut(&mut self) -> &mut MsgPalette {
        &mut self.msg
    }
}
