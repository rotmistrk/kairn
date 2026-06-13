//! Theme toggle and syntax theme commands.

use txv_core::palette::{detect_system_theme, ThemeMode};
use txv_core::program::CommandContext;

use crate::app_state::AppState;
use crate::desktop::SlotId;
use crate::handler::downcast_desktop;
use crate::views::editor::EditorView;

pub fn handle_toggle_theme(ctx: &mut CommandContext, state: &mut AppState) {
    let arg = ctx.data().as_ref().and_then(|d| d.downcast_ref::<String>()).cloned();
    let Some(ref ts) = state.theme_state else {
        return;
    };
    let mut ts = ts.borrow_mut();
    match arg.as_deref() {
        Some("dark") => {
            ts.mode = ThemeMode::Dark;
            ts.active = ts.dark.clone();
            ts.apply();
        }
        Some("light") => {
            ts.mode = ThemeMode::Light;
            ts.active = ts.light.clone();
            ts.apply();
        }
        Some("auto") => {
            let detected = detect_system_theme();
            ts.mode = detected;
            ts.active = match detected {
                ThemeMode::Light => ts.light.clone(),
                _ => ts.dark.clone(),
            };
            ts.apply();
        }
        _ => ts.toggle(),
    }
}

pub fn handle_set_syntax_theme(ctx: &mut CommandContext, state: &mut AppState) {
    let name = {
        let Some(n) = ctx.data().as_ref().and_then(|d| d.downcast_ref::<String>()) else {
            return;
        };
        n.clone()
    };
    let is_light = state
        .theme_state
        .as_ref()
        .map(|ts| ts.borrow().mode() == ThemeMode::Light)
        .unwrap_or(false);
    if is_light {
        state.settings.theme_syntax_light = name.clone();
    } else {
        state.settings.theme_syntax_dark = name.clone();
    }
    let Some(desktop) = downcast_desktop(ctx.desktop_mut()) else {
        return;
    };
    for slot in [SlotId::Center, SlotId::Tools] {
        let Some(panel) = desktop.panel_mut(slot as usize) else {
            continue;
        };
        for i in 0..panel.tab_count() {
            let editor = panel
                .view_at_mut(i)
                .and_then(|v| v.as_any_mut())
                .and_then(|a| a.downcast_mut::<EditorView>());
            if let Some(editor) = editor {
                editor.set_syntax_theme(&name);
            }
        }
    }
}

pub fn handle_set_glyphs(ctx: &mut CommandContext, state: &mut AppState) {
    use txv_core::glyphs::{set_glyphs, GlyphSet, GlyphTier};
    let Some(g) = ctx.data().as_ref().and_then(|d| d.downcast_ref::<String>()) else {
        return;
    };
    let tier = match g.as_str() {
        "ascii" => GlyphTier::Ascii,
        "utf" => GlyphTier::Unicode,
        "nerd" => GlyphTier::Nerd,
        _ => return,
    };
    set_glyphs(GlyphSet::from_tier(tier));
    state.settings.theme_glyphs = g.clone();
}
