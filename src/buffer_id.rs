//! BufferId — unique identifier for a buffer.

/// Unique identifier for a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId(u64);

impl BufferId {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for BufferId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "buf#{}", self.0)
    }
}
