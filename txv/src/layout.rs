// Layout engine — Rect::split with constraints.

/// Split direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// Size specification for a layout constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    /// Exact number of cells.
    Fixed(u16),
    /// Percentage of parent (0–100).
    Percent(u16),
    /// Take all remaining space.
    Fill,
}

/// A layout constraint with size, min, and max.
#[derive(Clone, Copy, Debug)]
pub struct Constraint {
    pub size: Size,
    pub min: u16,
    pub max: u16,
}

/// A rectangle in the terminal grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    /// Split this rect into sub-rects along a direction.
    pub fn split(&self, dir: Direction, constraints: &[Constraint]) -> Vec<Rect> {
        if constraints.is_empty() {
            return Vec::new();
        }
        let total = match dir {
            Direction::Horizontal => self.w,
            Direction::Vertical => self.h,
        };
        let mut sizes = vec![0u16; constraints.len()];
        let mut remaining = total;

        // Pass 1: allocate Fixed sizes
        for (i, c) in constraints.iter().enumerate() {
            if let Size::Fixed(v) = c.size {
                sizes[i] = v.clamp(c.min, c.max).min(remaining);
                remaining = remaining.saturating_sub(sizes[i]);
            }
        }

        // Pass 2: allocate Percent sizes from remaining
        let percent_base = remaining;
        for (i, c) in constraints.iter().enumerate() {
            if let Size::Percent(pct) = c.size {
                let v = (percent_base as u32 * pct.min(100) as u32 / 100) as u16;
                sizes[i] = v.clamp(c.min, c.max).min(remaining);
                remaining = remaining.saturating_sub(sizes[i]);
            }
        }

        // Pass 3: distribute remaining to Fill entries
        let fill_count = constraints.iter().filter(|c| c.size == Size::Fill).count() as u16;
        if fill_count > 0 {
            let per_fill = remaining / fill_count;
            let mut extra = remaining % fill_count;
            for (i, c) in constraints.iter().enumerate() {
                if c.size == Size::Fill {
                    let mut v = per_fill;
                    if extra > 0 {
                        v += 1;
                        extra -= 1;
                    }
                    sizes[i] = v.clamp(c.min, c.max);
                }
            }
        }

        // Pass 4: if total exceeds available, shrink from last to first
        let mut sum: u16 = sizes.iter().sum();
        if sum > total {
            for i in (0..sizes.len()).rev() {
                if sum <= total {
                    break;
                }
                let excess = sum - total;
                let can_shrink = sizes[i].saturating_sub(constraints[i].min);
                let shrink = excess.min(can_shrink);
                sizes[i] -= shrink;
                sum -= shrink;
            }
        }

        // Pass 5: if total < available, expand Fill entries
        sum = sizes.iter().sum();
        if sum < total {
            let deficit = total - sum;
            let fills: Vec<usize> = constraints
                .iter()
                .enumerate()
                .filter(|(_, c)| c.size == Size::Fill)
                .map(|(i, _)| i)
                .collect();
            if !fills.is_empty() {
                let per = deficit / fills.len() as u16;
                let mut extra = deficit % fills.len() as u16;
                for &i in &fills {
                    let mut add = per;
                    if extra > 0 {
                        add += 1;
                        extra -= 1;
                    }
                    sizes[i] = (sizes[i] + add).min(constraints[i].max);
                }
            }
        }

        // Build rects
        let mut pos = match dir {
            Direction::Horizontal => self.x,
            Direction::Vertical => self.y,
        };
        sizes
            .iter()
            .map(|&s| {
                let r = match dir {
                    Direction::Horizontal => Rect {
                        x: pos,
                        y: self.y,
                        w: s,
                        h: self.h,
                    },
                    Direction::Vertical => Rect {
                        x: self.x,
                        y: pos,
                        w: self.w,
                        h: s,
                    },
                };
                pos += s;
                r
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: 80,
            h: 24,
        }
    }

    #[test]
    fn split_fixed_and_fill() {
        let rects = area().split(
            Direction::Horizontal,
            &[
                Constraint {
                    size: Size::Fixed(20),
                    min: 10,
                    max: 40,
                },
                Constraint {
                    size: Size::Fill,
                    min: 10,
                    max: u16::MAX,
                },
            ],
        );
        assert_eq!(
            rects[0],
            Rect {
                x: 0,
                y: 0,
                w: 20,
                h: 24
            }
        );
        assert_eq!(
            rects[1],
            Rect {
                x: 20,
                y: 0,
                w: 60,
                h: 24
            }
        );
    }

    #[test]
    fn split_vertical() {
        let rects = area().split(
            Direction::Vertical,
            &[
                Constraint {
                    size: Size::Fixed(5),
                    min: 3,
                    max: 10,
                },
                Constraint {
                    size: Size::Fill,
                    min: 5,
                    max: u16::MAX,
                },
            ],
        );
        assert_eq!(
            rects[0],
            Rect {
                x: 0,
                y: 0,
                w: 80,
                h: 5
            }
        );
        assert_eq!(
            rects[1],
            Rect {
                x: 0,
                y: 5,
                w: 80,
                h: 19
            }
        );
    }

    #[test]
    fn split_percent() {
        let rects = area().split(
            Direction::Horizontal,
            &[
                Constraint {
                    size: Size::Percent(25),
                    min: 0,
                    max: u16::MAX,
                },
                Constraint {
                    size: Size::Fill,
                    min: 0,
                    max: u16::MAX,
                },
            ],
        );
        assert_eq!(rects[0].w, 20);
        assert_eq!(rects[1].w, 60);
    }

    #[test]
    fn split_multiple_fills() {
        let rects = area().split(
            Direction::Horizontal,
            &[
                Constraint {
                    size: Size::Fill,
                    min: 0,
                    max: u16::MAX,
                },
                Constraint {
                    size: Size::Fill,
                    min: 0,
                    max: u16::MAX,
                },
            ],
        );
        assert_eq!(rects[0].w, 40);
        assert_eq!(rects[1].w, 40);
    }

    #[test]
    fn split_min_clamping() {
        let small = Rect {
            x: 0,
            y: 0,
            w: 20,
            h: 10,
        };
        let rects = small.split(
            Direction::Horizontal,
            &[
                Constraint {
                    size: Size::Fixed(5),
                    min: 5,
                    max: 10,
                },
                Constraint {
                    size: Size::Fixed(5),
                    min: 5,
                    max: 10,
                },
                Constraint {
                    size: Size::Fill,
                    min: 5,
                    max: u16::MAX,
                },
            ],
        );
        assert!(rects[0].w >= 5);
        assert!(rects[1].w >= 5);
        assert!(rects[2].w >= 5);
    }

    #[test]
    fn split_max_clamping() {
        let rects = area().split(
            Direction::Horizontal,
            &[
                Constraint {
                    size: Size::Fixed(50),
                    min: 0,
                    max: 30,
                },
                Constraint {
                    size: Size::Fill,
                    min: 0,
                    max: u16::MAX,
                },
            ],
        );
        assert_eq!(rects[0].w, 30);
        assert_eq!(rects[1].w, 50);
    }

    #[test]
    fn split_empty_constraints() {
        let rects = area().split(Direction::Horizontal, &[]);
        assert!(rects.is_empty());
    }

    #[test]
    fn split_zero_size_area() {
        let zero = Rect {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        };
        let rects = zero.split(
            Direction::Horizontal,
            &[Constraint {
                size: Size::Fill,
                min: 0,
                max: u16::MAX,
            }],
        );
        assert_eq!(rects[0].w, 0);
    }

    #[test]
    fn split_three_panel_layout() {
        // Simulate kairn's wide layout: tree(20) | editor(fill) | terminal(fill)
        let rects = area().split(
            Direction::Horizontal,
            &[
                Constraint {
                    size: Size::Fixed(20),
                    min: 10,
                    max: 40,
                },
                Constraint {
                    size: Size::Fill,
                    min: 20,
                    max: u16::MAX,
                },
                Constraint {
                    size: Size::Fill,
                    min: 15,
                    max: u16::MAX,
                },
            ],
        );
        assert_eq!(rects[0].w, 20);
        assert_eq!(rects[1].w, 30);
        assert_eq!(rects[2].w, 30);
        assert_eq!(rects[0].w + rects[1].w + rects[2].w, 80);
    }

    #[test]
    fn split_fixed_percent_fill() {
        let rects = area().split(
            Direction::Horizontal,
            &[
                Constraint {
                    size: Size::Fixed(10),
                    min: 0,
                    max: u16::MAX,
                },
                Constraint {
                    size: Size::Percent(50),
                    min: 0,
                    max: u16::MAX,
                },
                Constraint {
                    size: Size::Fill,
                    min: 0,
                    max: u16::MAX,
                },
            ],
        );
        assert_eq!(rects[0].w, 10);
        // 50% of remaining 70 = 35
        assert_eq!(rects[1].w, 35);
        assert_eq!(rects[2].w, 35);
    }
}
