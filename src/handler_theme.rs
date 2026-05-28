//! Theme toggle and syntax theme commands.

use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::desktop::SlotId;
use crate::handler::downcast_desktop;
use crate::views::editor::EditorView;

pub fn handle_toggle_theme(ctx: &mut CommandContext, state: &mut AppState) {
    let arg = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()).cloned();
    let Some(ref ts) = state.theme_state else {
        return;
    };
    let mut ts = ts.borrow_mut();
    match arg.as_deref() {
        Some("dark") => {
            ts.mode = txv_core::palette::ThemeMode::Dark;
            ts.active = ts.dark.clone();
            ts.apply();
        }
        Some("light") => {
            ts.mode = txv_core::palette::ThemeMode::Light;
            ts.active = ts.light.clone();
            ts.apply();
        }
        Some("auto") => {
            let detected = txv_core::palette::detect_system_theme();
            ts.mode = detected;
            ts.active = match detected {
                txv_core::palette::ThemeMode::Light => ts.light.clone(),
                _ => ts.dark.clone(),
            };
            ts.apply();
        }
        _ => ts.toggle(),
    }
}

pub fn handle_set_syntax_theme(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(name) = ctx.data.as_ref().and_then(|d| d.downcast_ref::<String>()) else {
        return;
    };
    let is_light = state
        .theme_state
        .as_ref()
        .map(|ts| ts.borrow().mode == txv_core::palette::ThemeMode::Light)
        .unwrap_or(false);
    if is_light {
        state.settings.theme_syntax_light = name.clone();
    } else {
        state.settings.theme_syntax_dark = name.clone();
    }
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    for slot in [SlotId::Center, SlotId::Tools] {
        let Some(panel) = desktop.panel_mut(slot as usize) else {
            return;
        };
        for i in 0..panel.tab_count() {
            if let Some(view) = panel.view_at_mut(i) {
                if let Some(any) = view.as_any_mut() {
                    if let Some(editor) = any.downcast_mut::<EditorView>() {
                        editor.set_syntax_theme(name);
                    }
                }
            }
        }
    }
}
