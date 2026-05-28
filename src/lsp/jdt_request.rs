//! JDT class file content request.

/// Request for jdt:// class file contents from jdtls.
#[derive(Debug, Clone)]
pub(crate) struct JdtRequest {
    pub(crate) uri: String,
    pub(crate) line: u32,
    pub(crate) character: u32,
}
