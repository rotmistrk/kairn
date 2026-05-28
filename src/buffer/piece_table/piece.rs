//! Piece — a span referencing original or add buffer.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in crate::buffer) enum Source {
    Original,
    Add,
}

#[derive(Clone, Debug)]
pub(in crate::buffer) struct Piece {
    pub(crate) source: Source,
    pub(crate) start: usize,
    pub(crate) len: usize,
}
