//! NumericScale — per-column display multiplier for table view.

/// Scale factor applied to a numeric column for display.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ScaleUnit {
    Kilo, // k: ÷1000 or ×1000
    Kibi, // K: ÷1024 or ×1024
    Mega, // m: ÷10⁶ or ×10⁶
    Mebi, // M: ÷2²⁰ or ×2²⁰
    Giga, // g: ÷10⁹ or ×10⁹
    Gibi, // G: ÷2³⁰ or ×2³⁰
    Tera, // t: ÷10¹² or ×10¹²
    Tebi, // T: ÷2⁴⁰ or ×2⁴⁰
    Peta, // p: ÷10¹⁵ or ×10¹⁵
    Pebi, // P: ÷2⁵⁰ or ×2⁵⁰
    Exa,  // e: ÷10¹⁸ or ×10¹⁸
    Exbi, // E: ÷2⁶⁰ or ×2⁶⁰
}

/// Per-column numeric display state.
#[derive(Clone, Debug, Default)]
pub(crate) struct NumericScale {
    pub(crate) unit: Option<ScaleUnit>,
    /// False = divide (large→small), True = multiply (small→large suffix).
    pub(crate) inverted: bool,
}

impl NumericScale {
    pub(crate) fn toggle_unit(&mut self, unit: ScaleUnit) {
        if self.unit == Some(unit) {
            self.unit = None;
        } else {
            self.unit = Some(unit);
        }
    }

    pub(crate) fn toggle_direction(&mut self) {
        self.inverted = !self.inverted;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.unit.is_some()
    }

    /// Apply scale to a raw f64 value. Returns (scaled_value, suffix).
    pub(crate) fn apply(&self, value: f64) -> (f64, &'static str) {
        let Some(unit) = self.unit else {
            return (value, "");
        };
        let (divisor, suffix_normal, suffix_inv) = unit_info(unit);
        if self.inverted {
            (value * divisor, suffix_inv)
        } else {
            (value / divisor, suffix_normal)
        }
    }
}

fn unit_info(unit: ScaleUnit) -> (f64, &'static str, &'static str) {
    match unit {
        ScaleUnit::Kilo => (1e3, "k", "m"),
        ScaleUnit::Kibi => (1024.0, "Ki", ""),
        ScaleUnit::Mega => (1e6, "M", "µ"),
        ScaleUnit::Mebi => (1_048_576.0, "Mi", ""),
        ScaleUnit::Giga => (1e9, "G", "n"),
        ScaleUnit::Gibi => (1_073_741_824.0, "Gi", ""),
        ScaleUnit::Tera => (1e12, "T", "p"),
        ScaleUnit::Tebi => (2.0_f64.powi(40), "Ti", ""),
        ScaleUnit::Peta => (1e15, "P", "f"),
        ScaleUnit::Pebi => (2.0_f64.powi(50), "Pi", ""),
        ScaleUnit::Exa => (1e18, "E", "a"),
        ScaleUnit::Exbi => (2.0_f64.powi(60), "Ei", ""),
    }
}

/// Map a key char to a ScaleUnit.
pub(crate) fn key_to_unit(ch: char) -> Option<ScaleUnit> {
    match ch {
        'k' => Some(ScaleUnit::Kilo),
        'K' => Some(ScaleUnit::Kibi),
        'm' => Some(ScaleUnit::Mega),
        'M' => Some(ScaleUnit::Mebi),
        'g' => Some(ScaleUnit::Giga),
        'G' => Some(ScaleUnit::Gibi),
        't' => Some(ScaleUnit::Tera),
        'T' => Some(ScaleUnit::Tebi),
        'p' => Some(ScaleUnit::Peta),
        'P' => Some(ScaleUnit::Pebi),
        'e' => Some(ScaleUnit::Exa),
        'E' => Some(ScaleUnit::Exbi),
        _ => None,
    }
}
