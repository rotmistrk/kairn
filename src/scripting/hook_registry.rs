//! Registry of all hooks, fired in declaration order.

use regex::Regex;

use super::hooks::{CompiledFilter, Hook, HookEvent};

/// Registry of all hooks, fired in declaration order.
#[derive(Default)]
pub struct HookRegistry {
    hooks: Vec<Hook>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Add a hook. Filter is compiled as regex for char/word events, millis for idle.
    pub fn add(&mut self, event: HookEvent, filter: Option<&str>, body: String) -> Result<(), String> {
        let compiled = match filter {
            None => None,
            Some(f) => match &event {
                HookEvent::Idle => {
                    let ms = f.parse::<u64>().map_err(|e| format!("invalid idle ms: {e}"))?;
                    Some(CompiledFilter::Millis(ms))
                }
                _ => {
                    let re = Regex::new(f).map_err(|e| format!("invalid filter regex: {e}"))?;
                    Some(CompiledFilter::Regex(re))
                }
            },
        };
        self.hooks.push(Hook {
            event,
            filter: compiled,
            body,
        });
        Ok(())
    }

    /// Remove all hooks for a given event.
    pub fn remove(&mut self, event: &HookEvent) {
        self.hooks.retain(|h| &h.event != event);
    }

    /// List hooks, optionally filtered by event.
    pub fn list(&self, event: Option<&HookEvent>) -> Vec<String> {
        self.hooks
            .iter()
            .filter(|h| event.is_none() || event == Some(&h.event))
            .map(|h| format!("{}: {}", h.event.as_str(), h.body))
            .collect()
    }

    /// Fire hooks for an event with context. Returns scripts to execute.
    pub fn fire(&self, event: &HookEvent, context: &str) -> Vec<String> {
        self.hooks
            .iter()
            .filter(|h| &h.event == event)
            .filter(|h| matches_filter(&h.filter, context))
            .map(|h| h.body.clone())
            .collect()
    }
}

fn matches_filter(filter: &Option<CompiledFilter>, context: &str) -> bool {
    match filter {
        None => true,
        Some(CompiledFilter::Regex(re)) => re.is_match(context),
        Some(CompiledFilter::Millis(_)) => true, // Caller checks timing
    }
}
