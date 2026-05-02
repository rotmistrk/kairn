//! Single-row progress bar — determinate and indeterminate modes.

use txv::cell::Style;
use txv::surface::Surface;

/// Progress bar mode.
#[derive(Clone, Debug)]
pub enum ProgressMode {
    /// Known progress: 0.0 to 1.0.
    Determinate(f64),
    /// Unknown progress: animated bounce position (0-based tick).
    Indeterminate(usize),
}

/// A single-row progress bar.
pub struct ProgressBar {
    mode: ProgressMode,
    /// Label shown inside or beside the bar.
    pub label: String,
    /// Style for the filled portion.
    pub filled_style: Style,
    /// Style for the empty portion.
    pub empty_style: Style,
    /// Style for the label text.
    pub label_style: Style,
    /// Character for the filled portion.
    pub filled_char: char,
    /// Character for the empty portion.
    pub empty_char: char,
}

impl ProgressBar {
    /// Create a determinate progress bar at 0%.
    pub fn new() -> Self {
        Self {
            mode: ProgressMode::Determinate(0.0),
            label: String::new(),
            filled_style: Style {
                attrs: txv::cell::Attrs {
                    reverse: true,
                    ..txv::cell::Attrs::default()
                },
                ..Style::default()
            },
            empty_style: Style::default(),
            label_style: Style::default(),
            filled_char: '█',
            empty_char: '░',
        }
    }

    /// Create an indeterminate progress bar.
    pub fn indeterminate() -> Self {
        let mut pb = Self::new();
        pb.mode = ProgressMode::Indeterminate(0);
        pb
    }

    /// Set determinate progress (clamped to 0.0..=1.0).
    pub fn set_progress(&mut self, progress: f64) {
        self.mode = ProgressMode::Determinate(progress.clamp(0.0, 1.0));
    }

    /// Get current progress (0.0..=1.0), or None if indeterminate.
    pub fn progress(&self) -> Option<f64> {
        match self.mode {
            ProgressMode::Determinate(p) => Some(p),
            ProgressMode::Indeterminate(_) => None,
        }
    }

    /// Advance the indeterminate animation by one tick.
    /// No-op for determinate bars.
    pub fn tick(&mut self) {
        if let ProgressMode::Indeterminate(ref mut t) = self.mode {
            *t = t.wrapping_add(1);
        }
    }

    /// Set the mode directly.
    pub fn set_mode(&mut self, mode: ProgressMode) {
        self.mode = mode;
    }

    /// Render the progress bar into a surface (single row).
    pub fn render(&self, surface: &mut Surface<'_>) {
        let w = surface.width();
        if w == 0 {
            return;
        }

        match &self.mode {
            ProgressMode::Determinate(progress) => {
                let filled = (*progress * w as f64).round() as u16;
                surface.hline(0, 0, filled, self.filled_char, self.filled_style);
                if filled < w {
                    surface.hline(filled, 0, w - filled, self.empty_char, self.empty_style);
                }
            }
            ProgressMode::Indeterminate(tick) => {
                surface.hline(0, 0, w, self.empty_char, self.empty_style);
                let bounce_w = 3u16.min(w);
                let range = (w as usize).saturating_sub(bounce_w as usize).max(1);
                let cycle = range * 2;
                let pos = tick % cycle;
                let col = if pos < range { pos } else { cycle - pos };
                surface.hline(col as u16, 0, bounce_w, self.filled_char, self.filled_style);
            }
        }

        // Overlay label centered
        if !self.label.is_empty() {
            let lw = txv::text::display_width(&self.label) as u16;
            let x = w.saturating_sub(lw) / 2;
            surface.print(x, 0, &self.label, self.label_style);
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use txv::cell::ColorMode;
    use txv::screen::Screen;

    fn render_bar(pb: &ProgressBar, width: u16) -> String {
        let mut screen = Screen::with_color_mode(width, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            pb.render(&mut s);
        }
        screen.to_text().trim_end_matches('\n').to_string()
    }

    #[test]
    fn new_at_zero() {
        let pb = ProgressBar::new();
        assert_eq!(pb.progress(), Some(0.0));
    }

    #[test]
    fn set_progress_clamps() {
        let mut pb = ProgressBar::new();
        pb.set_progress(1.5);
        assert_eq!(pb.progress(), Some(1.0));
        pb.set_progress(-0.5);
        assert_eq!(pb.progress(), Some(0.0));
    }

    #[test]
    fn determinate_zero() {
        let pb = ProgressBar::new();
        let text = render_bar(&pb, 10);
        // All empty chars
        assert_eq!(text.chars().filter(|&c| c == '░').count(), 10);
    }

    #[test]
    fn determinate_full() {
        let mut pb = ProgressBar::new();
        pb.set_progress(1.0);
        let text = render_bar(&pb, 10);
        assert_eq!(text.chars().filter(|&c| c == '█').count(), 10);
    }

    #[test]
    fn determinate_half() {
        let mut pb = ProgressBar::new();
        pb.set_progress(0.5);
        let text = render_bar(&pb, 10);
        assert_eq!(text.chars().filter(|&c| c == '█').count(), 5);
        assert_eq!(text.chars().filter(|&c| c == '░').count(), 5);
    }

    #[test]
    fn indeterminate_renders() {
        let pb = ProgressBar::indeterminate();
        assert!(pb.progress().is_none());
        let text = render_bar(&pb, 20);
        // Should have some filled and some empty
        assert!(text.contains('█'));
        assert!(text.contains('░'));
    }

    #[test]
    fn indeterminate_tick_advances() {
        let mut pb = ProgressBar::indeterminate();
        let text1 = render_bar(&pb, 20);
        pb.tick();
        pb.tick();
        pb.tick();
        let text2 = render_bar(&pb, 20);
        // Position should differ after ticks
        assert_ne!(text1, text2);
    }

    #[test]
    fn tick_noop_on_determinate() {
        let mut pb = ProgressBar::new();
        pb.set_progress(0.5);
        pb.tick();
        assert_eq!(pb.progress(), Some(0.5));
    }

    #[test]
    fn label_renders_centered() {
        let mut pb = ProgressBar::new();
        pb.set_progress(0.5);
        pb.label = "50%".into();
        let text = render_bar(&pb, 20);
        assert!(text.contains("50%"));
    }

    #[test]
    fn zero_width_no_panic() {
        let pb = ProgressBar::new();
        let mut screen = Screen::with_color_mode(0, 1, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            pb.render(&mut s);
        }
        // Just shouldn't panic
    }

    #[test]
    fn indeterminate_bounces() {
        let mut pb = ProgressBar::indeterminate();
        let w = 10u16;
        // Collect positions over a full cycle
        let mut positions = Vec::new();
        for _ in 0..20 {
            let mut screen = Screen::with_color_mode(w, 1, ColorMode::Rgb);
            {
                let mut s = screen.full_surface();
                pb.render(&mut s);
            }
            // Find first filled char position
            let pos = (0..w).find(|&col| screen.cell(col, 0).ch == '█');
            positions.push(pos);
            pb.tick();
        }
        // Should have varying positions (bouncing)
        let unique: std::collections::HashSet<_> = positions.iter().collect();
        assert!(unique.len() > 1);
    }
}
