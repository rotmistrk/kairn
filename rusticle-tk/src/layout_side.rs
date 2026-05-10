//! Side specification for pack layout.

/// Side specification from the script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Horizontal split, widget on left.
    Left,
    /// Horizontal split, widget on right.
    Right,
    /// Vertical split, widget on top.
    Top,
    /// Vertical split, widget on bottom.
    Bottom,
    /// Takes remaining space.
    Fill,
}

impl Side {
    /// Parse a side string.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "top" => Ok(Self::Top),
            "bottom" => Ok(Self::Bottom),
            "fill" => Ok(Self::Fill),
            _ => Err(format!("unknown side: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_parse() {
        assert_eq!(Side::parse("left"), Ok(Side::Left));
        assert_eq!(Side::parse("right"), Ok(Side::Right));
        assert_eq!(Side::parse("top"), Ok(Side::Top));
        assert_eq!(Side::parse("bottom"), Ok(Side::Bottom));
        assert_eq!(Side::parse("fill"), Ok(Side::Fill));
        assert!(Side::parse("center").is_err());
    }
}
