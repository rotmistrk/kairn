//! Configurable palette — wraps a base palette with user overrides.

use std::sync::Arc;

use txv_core::cell::Style;
use txv_core::palette::style_id::StyleId;
use txv_core::palette::style_palette::StylePalette;
use txv_core::palette::{Base, Chrome, Interactive, Palette, Popup, State};

/// A palette that wraps a base and overrides specific styles from config.
pub struct CustomPalette {
    base: Arc<dyn Palette>,
    overrides: [Option<Style>; StyleId::COUNT],
}

impl CustomPalette {
    pub fn new(base: Arc<dyn Palette>) -> Self {
        Self {
            base,
            overrides: [None; StyleId::COUNT],
        }
    }

    pub fn set_override(&mut self, id: StyleId, style: Style) {
        self.overrides[id as usize] = Some(style);
    }

    fn get(&self, id: StyleId) -> Option<Style> {
        self.overrides[id as usize]
    }
}

impl Palette for CustomPalette {
    fn base(&self) -> &dyn Base {
        self
    }
    fn interactive(&self) -> &dyn Interactive {
        self
    }
    fn chrome(&self) -> &dyn Chrome {
        self
    }
    fn popup(&self) -> &dyn Popup {
        self
    }
    fn state(&self) -> &dyn State {
        self
    }
}

impl StylePalette for CustomPalette {
    fn style(&self, id: StyleId) -> Style {
        if let Some(s) = self.get(id) {
            return s;
        }
        // Delegate to base via old trait
        match id {
            StyleId::StatusBar => self.base.chrome().status_bar(),
            StyleId::StatusBarModal => self.base.chrome().status_bar_modal(),
            StyleId::InputCursor => self.base.interactive().input_cursor(),
            StyleId::Dim => self.base.base().dim(),
            StyleId::Text => self.base.base().text(),
            StyleId::Bright => self.base.base().bright(),
            StyleId::Border => self.base.base().border(),
            StyleId::Separator => self.base.base().separator(),
            StyleId::TreeDir => self.base.base().tree_dir(),
            StyleId::CursorFocused => self.base.interactive().cursor_focused(),
            StyleId::CursorUnfocused => self.base.interactive().cursor_unfocused(),
            StyleId::ChromeBar => self.base.chrome().bar(),
            StyleId::ScrollbarTrack => self.base.chrome().scrollbar_track(),
            StyleId::ScrollbarThumb => self.base.chrome().scrollbar_thumb(),
            StyleId::PopupBackground => self.base.popup().background(),
            StyleId::PopupBorder => self.base.popup().border(),
            StyleId::PopupSelected => self.base.popup().selected(),
            StyleId::StateError => self.base.state().error(),
            StyleId::StateWarning => self.base.state().warning(),
            StyleId::StateInfo => self.base.state().info(),
            StyleId::StateSuccess => self.base.state().success(),
            StyleId::StateHint => self.base.state().hint(),
            _ => Style::default(),
        }
    }
}

impl Base for CustomPalette {
    fn text(&self) -> Style {
        self.get(StyleId::Text).unwrap_or_else(|| self.base.base().text())
    }
    fn dim(&self) -> Style {
        self.get(StyleId::Dim).unwrap_or_else(|| self.base.base().dim())
    }
    fn bright(&self) -> Style {
        self.get(StyleId::Bright).unwrap_or_else(|| self.base.base().bright())
    }
    fn border(&self) -> Style {
        self.get(StyleId::Border).unwrap_or_else(|| self.base.base().border())
    }
    fn separator(&self) -> Style {
        self.get(StyleId::Separator)
            .unwrap_or_else(|| self.base.base().separator())
    }
    fn tree_dir(&self) -> Style {
        self.get(StyleId::TreeDir)
            .unwrap_or_else(|| self.base.base().tree_dir())
    }
}

impl Interactive for CustomPalette {
    fn cursor_focused(&self) -> Style {
        self.get(StyleId::CursorFocused)
            .unwrap_or_else(|| self.base.interactive().cursor_focused())
    }
    fn cursor_unfocused(&self) -> Style {
        self.get(StyleId::CursorUnfocused)
            .unwrap_or_else(|| self.base.interactive().cursor_unfocused())
    }
    fn input_cursor(&self) -> Style {
        self.get(StyleId::InputCursor)
            .unwrap_or_else(|| self.base.interactive().input_cursor())
    }
    fn edit_overlay(&self) -> Style {
        self.get(StyleId::EditOverlay)
            .unwrap_or_else(|| self.base.interactive().edit_overlay())
    }
    fn edit_selection(&self) -> Style {
        self.get(StyleId::EditSelection)
            .unwrap_or_else(|| self.base.interactive().edit_selection())
    }
    fn search_match(&self) -> Style {
        self.get(StyleId::SearchMatch)
            .unwrap_or_else(|| self.base.interactive().search_match())
    }
    fn visual_selection(&self) -> Style {
        self.get(StyleId::VisualSelection)
            .unwrap_or_else(|| self.base.interactive().visual_selection())
    }
    fn disabled(&self) -> Style {
        self.get(StyleId::Disabled)
            .unwrap_or_else(|| self.base.interactive().disabled())
    }
}

impl Chrome for CustomPalette {
    fn bar(&self) -> Style {
        self.get(StyleId::ChromeBar).unwrap_or_else(|| self.base.chrome().bar())
    }
    fn tab_focused(&self) -> Style {
        self.get(StyleId::TabFocused)
            .unwrap_or_else(|| self.base.chrome().tab_focused())
    }
    fn tab_focused_arrow(&self) -> Style {
        self.get(StyleId::TabFocusedArrow)
            .unwrap_or_else(|| self.base.chrome().tab_focused_arrow())
    }
    fn tab_focused_badge(&self) -> Style {
        self.get(StyleId::TabFocusedBadge)
            .unwrap_or_else(|| self.base.chrome().tab_focused_badge())
    }
    fn tab_active(&self) -> Style {
        self.get(StyleId::TabActive)
            .unwrap_or_else(|| self.base.chrome().tab_active())
    }
    fn tab_active_arrow(&self) -> Style {
        self.get(StyleId::TabActiveArrow)
            .unwrap_or_else(|| self.base.chrome().tab_active_arrow())
    }
    fn tab_active_badge(&self) -> Style {
        self.get(StyleId::TabActiveBadge)
            .unwrap_or_else(|| self.base.chrome().tab_active_badge())
    }
    fn tab_inactive(&self, distance: usize) -> Style {
        self.get(StyleId::TabInactive)
            .unwrap_or_else(|| self.base.chrome().tab_inactive(distance))
    }
    fn status_bar(&self) -> Style {
        self.get(StyleId::StatusBar)
            .unwrap_or_else(|| self.base.chrome().status_bar())
    }
    fn status_bar_modal(&self) -> Style {
        self.get(StyleId::StatusBarModal)
            .unwrap_or_else(|| self.base.chrome().status_bar_modal())
    }
    fn scrollbar_track(&self) -> Style {
        self.get(StyleId::ScrollbarTrack)
            .unwrap_or_else(|| self.base.chrome().scrollbar_track())
    }
    fn scrollbar_thumb(&self) -> Style {
        self.get(StyleId::ScrollbarThumb)
            .unwrap_or_else(|| self.base.chrome().scrollbar_thumb())
    }
}

impl Popup for CustomPalette {
    fn background(&self) -> Style {
        self.get(StyleId::PopupBackground)
            .unwrap_or_else(|| self.base.popup().background())
    }
    fn border(&self) -> Style {
        self.get(StyleId::PopupBorder)
            .unwrap_or_else(|| self.base.popup().border())
    }
    fn selected(&self) -> Style {
        self.get(StyleId::PopupSelected)
            .unwrap_or_else(|| self.base.popup().selected())
    }
    fn table_header(&self) -> Style {
        self.get(StyleId::PopupTableHeader)
            .unwrap_or_else(|| self.base.popup().table_header())
    }
}

impl State for CustomPalette {
    fn error(&self) -> Style {
        self.get(StyleId::StateError)
            .unwrap_or_else(|| self.base.state().error())
    }
    fn warning(&self) -> Style {
        self.get(StyleId::StateWarning)
            .unwrap_or_else(|| self.base.state().warning())
    }
    fn info(&self) -> Style {
        self.get(StyleId::StateInfo).unwrap_or_else(|| self.base.state().info())
    }
    fn success(&self) -> Style {
        self.get(StyleId::StateSuccess)
            .unwrap_or_else(|| self.base.state().success())
    }
    fn hint(&self) -> Style {
        self.get(StyleId::StateHint).unwrap_or_else(|| self.base.state().hint())
    }
}
