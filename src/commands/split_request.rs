//! Payload for CM_SPLIT.

#[derive(Debug, Clone)]
pub struct SplitRequest {
    pub(crate) vertical: bool,
    pub(crate) file: Option<String>,
}

impl SplitRequest {
    pub fn horizontal() -> Self {
        Self {
            vertical: false,
            file: None,
        }
    }
    pub fn vertical() -> Self {
        Self {
            vertical: true,
            file: None,
        }
    }
    pub fn vertical_with_file(file: String) -> Self {
        Self {
            vertical: true,
            file: Some(file),
        }
    }
    pub fn horizontal_with_file(file: String) -> Self {
        Self {
            vertical: false,
            file: Some(file),
        }
    }
    pub fn is_vertical(&self) -> bool {
        self.vertical
    }
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }
}
