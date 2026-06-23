//! Workspace-related state: broker, buffers, root directory, roots, settings.

use std::path::PathBuf;

use crate::broker::FileBroker;
use crate::buffer_registry::BufferRegistry;
use crate::settings::AppSettings;
use crate::workspace_roots::WorkspaceRoots;

/// Workspace state: file broker, buffers, root paths, and settings.
pub(crate) struct WorkspaceState {
    broker: FileBroker,
    buffers: BufferRegistry,
    root_dir: PathBuf,
    roots: WorkspaceRoots,
    settings: AppSettings,
}

impl WorkspaceState {
    pub(crate) fn new(root_dir: PathBuf, settings: AppSettings) -> Self {
        Self {
            broker: FileBroker::new(),
            buffers: BufferRegistry::new(),
            roots: WorkspaceRoots::new(root_dir.clone()),
            root_dir,
            settings,
        }
    }

    pub(crate) fn broker(&self) -> &FileBroker {
        &self.broker
    }

    pub(crate) fn broker_mut(&mut self) -> &mut FileBroker {
        &mut self.broker
    }

    pub(crate) fn buffers(&self) -> &BufferRegistry {
        &self.buffers
    }

    pub(crate) fn buffers_mut(&mut self) -> &mut BufferRegistry {
        &mut self.buffers
    }

    pub(crate) fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }

    pub(crate) fn set_root_dir(&mut self, dir: PathBuf) {
        self.root_dir = dir;
    }

    pub(crate) fn roots(&self) -> &WorkspaceRoots {
        &self.roots
    }

    pub(crate) fn roots_mut(&mut self) -> &mut WorkspaceRoots {
        &mut self.roots
    }

    pub(crate) fn settings(&self) -> &AppSettings {
        &self.settings
    }

    pub(crate) fn settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }
}
