//! Session persistence — save/restore workspace state.

mod restore;
mod save;
pub mod schema;

use std::path::Path;

use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::kiro_registry::KiroTabRegistry;
use schema::{SessionState, SESSION_VERSION};

pub use restore::{restore_kiro_tabs, restore_session, restore_tabs};

const STATE_FILE: &str = ".kairn.state";

/// Collect current state from the desktop and save to `.kairn.state`.
pub fn save_session(desktop: &mut TiledWorkspace, root_dir: &Path, kiro_registry: &KiroTabRegistry) {
    let state = save::collect_state(desktop, root_dir, kiro_registry);
    if state.editor_tabs.is_empty() && state.unfolded_dirs.is_empty() && state.kiro_sessions.is_empty() {
        return;
    }
    let path = root_dir.join(STATE_FILE);
    let json = match serde_json::to_string_pretty(&state) {
        Ok(j) => j,
        Err(e) => {
            log::warn!("session: failed to serialize: {e}");
            return;
        }
    };
    if let Err(e) = std::fs::write(&path, json) {
        log::warn!("session: failed to write {}: {e}", path.display());
    }
}

/// Load session state from `.kairn.state`. Returns None if missing/corrupt.
pub fn load_session(root_dir: &Path) -> Option<SessionState> {
    let path = root_dir.join(STATE_FILE);
    let content = std::fs::read_to_string(&path).ok()?;
    let state: SessionState = match serde_json::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            log::info!("session: ignoring corrupt state file: {e}");
            return None;
        }
    };
    if state.version != SESSION_VERSION && state.version != 1 {
        if state.version == 3 {
            // v3 had a save/restore order mismatch for proportions — clear them
            let mut state = state;
            state.wide_proportions.clear();
            state.narrow_proportions.clear();
            return Some(state);
        }
        log::info!("session: version mismatch (got {})", state.version);
        return None;
    }
    Some(state)
}
