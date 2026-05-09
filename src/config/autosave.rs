//! Timer-based autosave for modified buffers.

use std::time::{Duration, Instant};

/// Autosave timer state.
pub struct AutoSave {
    enabled: bool,
    interval: Duration,
    last_save: Instant,
}

impl AutoSave {
    /// Create a new autosave timer.
    pub fn new(enabled: bool, interval_secs: u32) -> Self {
        Self {
            enabled,
            interval: Duration::from_secs(interval_secs as u64),
            last_save: Instant::now(),
        }
    }

    /// Check if it's time to save. Returns true and resets timer if due.
    pub fn should_save(&mut self) -> bool {
        if !self.enabled {
            return false;
        }
        if self.last_save.elapsed() >= self.interval {
            self.last_save = Instant::now();
            return true;
        }
        false
    }

    /// Enable or disable autosave.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Update the interval.
    pub fn set_interval(&mut self, secs: u32) {
        self.interval = Duration::from_secs(secs as u64);
    }

    /// Reset the timer (e.g. after manual save).
    pub fn reset(&mut self) {
        self.last_save = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_never_fires() {
        let mut as_ = AutoSave::new(false, 1);
        // Even with zero interval, disabled means no save.
        as_.interval = Duration::ZERO;
        assert!(!as_.should_save());
    }

    #[test]
    fn fires_after_interval() {
        let mut as_ = AutoSave::new(true, 0);
        // Zero interval means immediate.
        assert!(as_.should_save());
    }

    #[test]
    fn reset_delays_next_save() {
        let mut as_ = AutoSave::new(true, 60);
        as_.reset();
        assert!(!as_.should_save());
    }
}
