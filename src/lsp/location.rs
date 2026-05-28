/// A location result from definition/references responses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub(crate) uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
}

impl Location {
    pub fn new(uri: impl Into<String>, line: u32, character: u32) -> Self {
        Self {
            uri: uri.into(),
            line,
            character,
        }
    }
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn line(&self) -> u32 {
        self.line
    }
    pub fn character(&self) -> u32 {
        self.character
    }
}
