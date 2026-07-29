//! FileRenameWatcher — detects file renames/moves and reports them.
//!
//! Watches the project root for rename events. When a file that matches
//! an open editor path is renamed, queues the (old_path, new_path) pair
//! for the main thread to process.

use std::mem;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use txv_core::run::Waker;

/// A queued rename event: (old_path, new_path).
pub(crate) type RenameEvent = (PathBuf, PathBuf);

/// Watches for file renames in the project root.
pub struct FileRenameWatcher {
    _watcher: RecommendedWatcher,
    events: Arc<Mutex<Vec<RenameEvent>>>,
}

impl FileRenameWatcher {
    /// Create a watcher on the project root. Returns None if creation fails.
    pub fn new(root: &std::path::Path, waker: Waker) -> Option<Self> {
        let events: Arc<Mutex<Vec<RenameEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let events_clone = events.clone();
        let pending_from: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            let Ok(event) = res else {
                return;
            };
            handle_notify_event(&event, &events_clone, &pending_from, &waker);
        })
        .ok()?;

        if watcher.watch(root, RecursiveMode::Recursive).is_err() {
            return None;
        }

        Some(Self {
            _watcher: watcher,
            events,
        })
    }

    /// Drain all pending rename events.
    pub fn drain(&self) -> Vec<RenameEvent> {
        self.events
            .lock()
            .ok()
            .map(|mut v| mem::take(&mut *v))
            .unwrap_or_default()
    }
}

fn handle_notify_event(
    event: &notify::Event,
    events: &Arc<Mutex<Vec<RenameEvent>>>,
    pending_from: &Arc<Mutex<Option<PathBuf>>>,
    waker: &Waker,
) {
    use notify::event::{ModifyKind, RenameMode};
    use notify::EventKind;

    match &event.kind {
        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            stash_rename_from(event, pending_from);
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
            complete_rename(event, events, pending_from, waker);
        }
        _ => {}
    }
}

fn stash_rename_from(event: &notify::Event, pending_from: &Arc<Mutex<Option<PathBuf>>>) {
    if let Some(path) = event.paths.first() {
        if let Ok(mut pf) = pending_from.lock() {
            *pf = Some(path.clone());
        }
    }
}

fn complete_rename(
    event: &notify::Event,
    events: &Arc<Mutex<Vec<RenameEvent>>>,
    pending_from: &Arc<Mutex<Option<PathBuf>>>,
    waker: &Waker,
) {
    let Some(new_path) = event.paths.first() else {
        return;
    };
    let old_path = pending_from.lock().ok().and_then(|mut pf| pf.take());
    if let Some(old) = old_path {
        if let Ok(mut ev) = events.lock() {
            ev.push((old, new_path.clone()));
        }
        waker.wake();
    }
}
