//! Registry of all hooks, fired in declaration order.

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

    /// Add a hook. Filter is a glob pattern (or millis for idle events).
    pub fn add(&mut self, event: HookEvent, filter: Option<&str>, body: String) -> Result<(), String> {
        let compiled = match filter {
            None => None,
            Some(f) => match &event {
                HookEvent::Idle => {
                    let ms = f.parse::<u64>().map_err(|e| format!("invalid idle ms: {e}"))?;
                    Some(CompiledFilter::Millis(ms))
                }
                _ => Some(CompiledFilter::Pattern(f.to_string())),
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
        Some(CompiledFilter::Pattern(pat)) => glob_match(pat, context),
        Some(CompiledFilter::Millis(_)) => true, // Caller checks timing
    }
}

/// Simple glob match: `*` matches any sequence, `?` matches one char, rest is literal.
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if !pattern.contains('*') && !pattern.contains('?') {
        return pattern == text;
    }
    let pat: Vec<_> = pattern.chars().collect();
    let txt: Vec<_> = text.chars().collect();
    glob_match_inner(&pat, &txt)
}

fn glob_match_inner(pattern: &[char], text: &[char]) -> bool {
    let (mut pi, mut ti) = (0, 0);
    let (mut star_pi, mut star_ti) = (usize::MAX, 0);
    while ti < text.len() {
        if pi < pattern.len() && (pattern[pi] == '?' || pattern[pi] == text[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < pattern.len() && pattern[pi] == '*' {
            star_pi = pi;
            star_ti = ti;
            pi += 1;
        } else if star_pi != usize::MAX {
            pi = star_pi + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }
    while pi < pattern.len() && pattern[pi] == '*' {
        pi += 1;
    }
    pi == pattern.len()
}
