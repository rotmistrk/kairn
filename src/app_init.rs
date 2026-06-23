//! App initialization — shared between main() and test harness.

use std::path::Path;

use txv_core::program::Program;
use txv_widgets::sidekick_manager::SidekickManager;

use crate::build_desktop::build_workspace;
use crate::completer::AppCompleter;
use crate::config::load_config;
use crate::handler::AppState;
use crate::settings::GitKeys;
use crate::startup::configure_app_state;
use crate::status::build_status_bar;
use crate::views::tree::FileTreeView;

/// Build the full application: AppState + Program, ready to run.
/// This is the single source of truth for app initialization.
pub fn build_app(root_dir: &Path) -> (Program, AppState) {
    let settings = load_config(root_dir);
    let git_keys = settings.git_keys().clone();
    let mut app_state = AppState::with_settings(root_dir.to_path_buf(), settings);
    configure_app_state(&mut app_state, root_dir);

    let mut desktop = build_workspace(root_dir, git_keys);
    desktop.set_wide_threshold(app_state.settings().layout_wide_threshold());
    desktop.set_focus_subpanels(true);
    apply_tree_icons(&mut desktop, &app_state);
    apply_tree_connectors(&mut desktop, &app_state);
    apply_todo_clipboard(&mut desktop, &app_state);
    let mut completer = AppCompleter::new(root_dir.to_path_buf(), app_state.scripting().command_list().clone());
    completer.set_lsp_languages(app_state.lsp_languages().clone());
    completer.set_roots(app_state.scripting().completer_roots().clone());
    let status = build_status_bar(
        &desktop,
        Box::new(completer),
        app_state.settings().clock_interval(),
        root_dir.to_path_buf(),
        app_state.settings().status_keys(),
        app_state.editor().clipboard().clone(),
        app_state.editor().command_history().clone(),
    );
    let mut program = Program::new(Box::new(status), Box::new(desktop));
    program.insert_named("sidekick", Box::new(SidekickManager::new()));
    // Collect live key binding descriptions for :help keys
    let bindings = program.status_bar().key_help();
    app_state.set_key_bindings(bindings);

    (program, app_state)
}

/// Build with custom git keys (for when config is already loaded externally).
pub fn build_app_with(root_dir: &Path, git_keys: GitKeys, app_state: &mut AppState) -> Program {
    let mut desktop = build_workspace(root_dir, git_keys);
    desktop.set_wide_threshold(app_state.settings().layout_wide_threshold());
    desktop.set_focus_subpanels(true);
    apply_tree_icons(&mut desktop, app_state);
    apply_tree_connectors(&mut desktop, app_state);

    let mut completer = AppCompleter::new(root_dir.to_path_buf(), app_state.scripting().command_list().clone());
    completer.set_lsp_languages(app_state.lsp_languages().clone());
    completer.set_roots(app_state.scripting().completer_roots().clone());
    let status = build_status_bar(
        &desktop,
        Box::new(completer),
        app_state.settings().clock_interval(),
        root_dir.to_path_buf(),
        app_state.settings().status_keys(),
        app_state.editor().clipboard().clone(),
        app_state.editor().command_history().clone(),
    );
    let mut program = Program::new(Box::new(status), Box::new(desktop));
    program.insert_named("sidekick", Box::new(SidekickManager::new()));
    program
}

fn apply_tree_icons(desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace, state: &AppState) {
    if !state.settings().tree_icons() {
        return;
    }
    let Some(panel) = desktop.panel_mut(0) else {
        return;
    };
    let Some(view) = panel.view_at_mut(0) else {
        return;
    };
    if let Some(tree) = view.as_any_mut().and_then(|a| a.downcast_mut::<FileTreeView>()) {
        tree.set_show_icons(true);
    }
}

fn apply_tree_connectors(desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace, state: &AppState) {
    if !state.settings().tree_connectors() {
        return;
    }
    let Some(panel) = desktop.panel_mut(0) else {
        return;
    };
    let Some(view) = panel.view_at_mut(0) else {
        return;
    };
    if let Some(tree) = view.as_any_mut().and_then(|a| a.downcast_mut::<FileTreeView>()) {
        tree.inner.set_show_connectors(true);
    }
}

fn apply_todo_clipboard(desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace, state: &AppState) {
    use crate::views::todo_tree::TodoTreeView;
    let Some(panel) = desktop.panel_mut(0) else {
        return;
    };
    for i in 0..panel.tab_count() {
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        if let Some(todo) = view.as_any_mut().and_then(|a| a.downcast_mut::<TodoTreeView>()) {
            todo.clipboard = Some(state.editor().clipboard().clone());
        }
    }
}
