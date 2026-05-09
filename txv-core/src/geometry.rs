//! Geometry primitives: Point and Rect.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x
            && p.y >= self.y
            && p.x < self.x.saturating_add(self.w)
            && p.y < self.y.saturating_add(self.h)
    }

    pub fn intersect(&self, other: Rect) -> Rect {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = self
            .x
            .saturating_add(self.w)
            .min(other.x.saturating_add(other.w));
        let y2 = self
            .y
            .saturating_add(self.h)
            .min(other.y.saturating_add(other.h));
        if x2 <= x1 || y2 <= y1 {
            Rect::default()
        } else {
            Rect::new(x1, y1, x2 - x1, y2 - y1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains() {
        let r = Rect::new(5, 5, 10, 10);
        assert!(r.contains(Point { x: 5, y: 5 }));
        assert!(r.contains(Point { x: 14, y: 14 }));
        assert!(!r.contains(Point { x: 15, y: 15 }));
        assert!(!r.contains(Point { x: 4, y: 5 }));
    }

    #[test]
    fn rect_intersect() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.intersect(b), Rect::new(5, 5, 5, 5));
    }

    #[test]
    fn rect_no_intersect() {
        let a = Rect::new(0, 0, 5, 5);
        let b = Rect::new(10, 10, 5, 5);
        assert!(a.intersect(b).is_empty());
    }
}
