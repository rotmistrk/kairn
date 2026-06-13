//! Runtime theme state — holds both palettes and supports hot-swap.

use std::sync::Arc;

use txv_core::palette::dark::DarkPalette;
use txv_core::palette::light::LightPalette;
use txv_core::palette::{detect_system_theme, set_palette, Palette, ThemeMode};

use crate::app_palette::{set_app_palette, AppPalette};

/// Runtime theme state — holds both palettes and supports hot-swap.
pub struct ThemeState {
    pub(crate) active: AppPalette,
    pub(crate) dark: AppPalette,
    pub(crate) light: AppPalette,
    pub(crate) mode: ThemeMode,
}

impl ThemeState {
    pub fn mode(&self) -> ThemeMode {
        self.mode
    }

    pub fn new(mode: ThemeMode) -> Self {
        let resolved = match mode {
            ThemeMode::Auto => detect_system_theme(),
            ref m => *m,
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
        let framework_pal: Arc<dyn Palette> = match self.mode {
            ThemeMode::Light => Arc::new(LightPalette),
            _ => Arc::new(DarkPalette),
        };
        set_palette(framework_pal);
        set_app_palette(&self.active);
    }
}

impl Default for ThemeState {
    fn default() -> Self {
        Self::new(ThemeMode::Auto)
    }
}
