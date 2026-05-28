//! Runtime theme state — holds both palettes and supports hot-swap.

use txv_core::palette::ThemeMode;

use crate::app_palette::{set_app_palette, AppPalette};

/// Runtime theme state — holds both palettes and supports hot-swap.
pub struct ThemeState {
    pub active: AppPalette,
    pub dark: AppPalette,
    pub light: AppPalette,
    pub mode: ThemeMode,
}

impl ThemeState {
    pub fn new(mode: ThemeMode) -> Self {
        let resolved = match mode {
            ThemeMode::Auto => txv_core::palette::detect_system_theme(),
            ref m => m.clone(),
        };
        let dark = AppPalette::dark();
        let light = AppPalette::light();
        let active = match resolved {
            ThemeMode::Light => light.clone(),
            _ => dark.clone(),
        };
        Self {
            active,
            dark,
            light,
            mode: resolved,
        }
    }

    pub fn toggle(&mut self) {
        match self.mode {
            ThemeMode::Dark => {
                self.mode = ThemeMode::Light;
                self.active = self.light.clone();
            }
            _ => {
                self.mode = ThemeMode::Dark;
                self.active = self.dark.clone();
            }
        }
        self.apply();
    }

    /// Apply the active palette to the global state.
    pub fn apply(&self) {
        let framework_pal: std::sync::Arc<dyn txv_core::palette::Palette> = match self.mode {
            ThemeMode::Light => std::sync::Arc::new(txv_core::palette::light::LightPalette),
            _ => std::sync::Arc::new(txv_core::palette::dark::DarkPalette),
        };
        txv_core::palette::set_palette(framework_pal);
        set_app_palette(&self.active);
    }
}

impl Default for ThemeState {
    fn default() -> Self {
        Self::new(ThemeMode::Auto)
    }
}
