//! Internal piece table operations (insert/delete at piece level).

use super::{Piece, PieceTable};

impl PieceTable {
    pub(super) fn insert_piece_at(&mut self, offset: usize, new_piece: Piece) {
        if self.pieces.is_empty() {
            self.pieces.push(new_piece);
            return;
        }
        let mut cumulative = 0;
        for i in 0..self.pieces.len() {
            let p = &self.pieces[i];
            if cumulative + p.len > offset || (cumulative + p.len == offset && i == self.pieces.len() - 1) {
                let local = offset - cumulative;
                if local == 0 {
                    self.pieces.insert(i, new_piece);
                } else if local == p.len {
                    self.pieces.insert(i + 1, new_piece);
                } else {
                    let left = Piece { source: p.source, start: p.start, len: local };
                    let right = Piece { source: p.source, start: p.start + local, len: p.len - local };
                    self.pieces.splice(i..=i, [left, new_piece, right]);
                }
                return;
            }
            cumulative += p.len;
        }
        self.pieces.push(new_piece);
    }

    pub(super) fn delete_range_internal(&mut self, start: usize, end: usize) {
        let mut new_pieces = Vec::new();
        let mut cumulative = 0;
        for p in &self.pieces {
            let p_start = cumulative;
            let p_end = cumulative + p.len;
            cumulative = p_end;

            if p_end <= start || p_start >= end {
                new_pieces.push(p.clone());
            } else {
                if p_start < start {
                    let keep = start - p_start;
                    new_pieces.push(Piece { source: p.source, start: p.start, len: keep });
                }
                if p_end > end {
                    let skip = end - p_start;
                    new_pieces.push(Piece { source: p.source, start: p.start + skip, len: p.len - skip });
                }
            }
        }
        self.pieces = new_pieces;
    }
}
