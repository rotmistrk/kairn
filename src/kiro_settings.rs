//! Settings for launching kiro-cli sessions.

/// Settings for launching kiro-cli sessions.
#[derive(Debug, Clone)]
pub struct KiroLaunchSettings {
    /// Base command as argv (e.g. ["kiro-cli", "chat", "--tui"]).
    pub(crate) cmd: Vec<String>,
    /// Extra args appended for first restored session (e.g. ["--resume"]).
    pub(crate) resume_first: Vec<String>,
    /// Extra args appended for remaining restored sessions (e.g. ["--resume-picker"]).
    pub(crate) resume_rest: Vec<String>,
}

impl Default for KiroLaunchSettings {
    fn default() -> Self {
        Self {
            cmd: vec!["kiro-cli".into(), "chat".into()],
            resume_first: vec!["--resume".into()],
            resume_rest: vec!["--resume-picker".into()],
        }
    }
}
