//! Secret fancy alphabet display modes — braille, fraktur, runic.
//!
//! Pure display transform: remaps ASCII during rendering without modifying the buffer.

/// Display mode for the secret alphabet feature.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub(crate) enum FancyAlpha {
    #[default]
    Normal,
    Braille,
    Fraktur,
    Runic,
    Tengwar,
}

impl FancyAlpha {
    /// Cycle to next mode.
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Normal => Self::Braille,
            Self::Braille => Self::Fraktur,
            Self::Fraktur => Self::Runic,
            Self::Runic => Self::Tengwar,
            Self::Tengwar => Self::Normal,
        }
    }

    /// Status bar indicator character.
    pub(crate) fn indicator(self) -> Option<char> {
        match self {
            Self::Normal => None,
            Self::Braille => Some('⠃'),
            Self::Fraktur => Some('𝔉'),
            Self::Runic => Some('ᚱ'),
            Self::Tengwar => Some('\u{E004}'),
        }
    }

    /// Remap a character for display. Returns the original if no mapping.
    pub(crate) fn remap(self, ch: char) -> char {
        match self {
            Self::Normal => ch,
            Self::Braille => remap_braille(ch),
            Self::Fraktur => remap_fraktur(ch),
            Self::Runic => remap_runic(ch),
            Self::Tengwar => remap_tengwar(ch),
        }
    }
}

fn remap_braille(ch: char) -> char {
    match ch {
        'a'..='z' => BRAILLE_LOWER[(ch as u8 - b'a') as usize],
        'A'..='Z' => BRAILLE_LOWER[(ch as u8 - b'A') as usize],
        '0'..='9' => BRAILLE_DIGITS[(ch as u8 - b'0') as usize],
        ' ' => '⠀', // braille blank
        '.' => '⠲',
        ',' => '⠂',
        '!' => '⠖',
        '?' => '⠦',
        '-' => '⠤',
        _ => ch,
    }
}

fn remap_fraktur(ch: char) -> char {
    match ch {
        'a'..='z' => FRAKTUR_LOWER[(ch as u8 - b'a') as usize],
        'A'..='Z' => FRAKTUR_UPPER[(ch as u8 - b'A') as usize],
        '0'..='9' => ch, // fraktur has no digit variants
        _ => ch,
    }
}

fn remap_runic(ch: char) -> char {
    match ch {
        'a'..='z' => RUNIC_MAP[(ch as u8 - b'a') as usize],
        'A'..='Z' => RUNIC_MAP[(ch as u8 - b'A') as usize],
        _ => ch,
    }
}

// Standard braille: a=⠁, b=⠃, c=⠉, ...
#[rustfmt::skip]
const BRAILLE_LOWER: [char; 26] = [
    '⠁', '⠃', '⠉', '⠙', '⠑', '⠋', '⠛', '⠓', '⠊', '⠚',
    '⠅', '⠇', '⠍', '⠝', '⠕', '⠏', '⠟', '⠗', '⠎', '⠞',
    '⠥', '⠧', '⠺', '⠭', '⠽', '⠵',
];

// Braille digits (uses letter values with dot-6: a=1, b=2, ... j=0)
const BRAILLE_DIGITS: [char; 10] = [
    '⠚', // 0 (j)
    '⠁', // 1 (a)
    '⠃', // 2 (b)
    '⠉', // 3 (c)
    '⠙', // 4 (d)
    '⠑', // 5 (e)
    '⠋', // 6 (f)
    '⠛', // 7 (g)
    '⠓', // 8 (h)
    '⠊', // 9 (i)
];

// Mathematical Fraktur: U+1D504 (upper), U+1D51E (lower)
// Some letters have special codepoints (C, H, I, R, Z)
#[rustfmt::skip]
const FRAKTUR_UPPER: [char; 26] = [
    '𝔄', '𝔅', 'ℭ', '𝔇', '𝔈', '𝔉', '𝔊', 'ℌ', 'ℑ', '𝔍',
    '𝔎', '𝔏', '𝔐', '𝔑', '𝔒', '𝔓', '𝔔', 'ℜ', '𝔖', '𝔗',
    '𝔘', '𝔙', '𝔚', '𝔛', '𝔜', 'ℨ',
];

#[rustfmt::skip]
const FRAKTUR_LOWER: [char; 26] = [
    '𝔞', '𝔟', '𝔠', '𝔡', '𝔢', '𝔣', '𝔤', '𝔥', '𝔦', '𝔧',
    '𝔨', '𝔩', '𝔪', '𝔫', '𝔬', '𝔭', '𝔮', '𝔯', '𝔰', '𝔱',
    '𝔲', '𝔳', '𝔴', '𝔵', '𝔶', '𝔷',
];

// Runic approximations (best-effort phonetic mapping)
const RUNIC_MAP: [char; 26] = [
    'ᚨ', // a - ansuz
    'ᛒ', // b - berkanan
    'ᚲ', // c - kaunan
    'ᛞ', // d - dagaz
    'ᛖ', // e - ehwaz
    'ᚠ', // f - fehu
    'ᚷ', // g - gebo
    'ᚺ', // h - hagalaz
    'ᛁ', // i - isaz
    'ᛃ', // j - jera
    'ᚲ', // k - kaunan (same as c)
    'ᛚ', // l - laguz
    'ᛗ', // m - mannaz
    'ᚾ', // n - naudiz
    'ᛟ', // o - othala
    'ᛈ', // p - perthro
    'ᚲ', // q - kaunan (approx)
    'ᚱ', // r - raido
    'ᛊ', // s - sowilo
    'ᛏ', // t - tiwaz
    'ᚢ', // u - uruz
    'ᚹ', // v - wunjo (approx)
    'ᚹ', // w - wunjo
    'ᛉ', // x - algiz (approx)
    'ᛃ', // y - jera (approx)
    'ᛉ', // z - algiz
];

fn remap_tengwar(ch: char) -> char {
    match ch {
        'a'..='z' => TENGWAR_MAP[(ch as u8 - b'a') as usize],
        'A'..='Z' => TENGWAR_MAP[(ch as u8 - b'A') as usize],
        _ => ch,
    }
}

// Tengwar phonetic approximation (CSUR PUA mapping U+E000+)
// These render with Tengwar-capable fonts (Nerd Font, Tengwar Annatar)
// Phonetic: tinco=t, parma=p, calma=c, quesse=q, ando=d, umbar=b, anga=g, ungwe=ng
// Falls back to boxes on terminals without the font — that's the "secret" part!
const TENGWAR_MAP: [char; 26] = [
    '\u{E003}', // a - anna
    '\u{E010}', // b - umbar
    '\u{E006}', // c - calma
    '\u{E00C}', // d - ando
    '\u{E001}', // e - ore
    '\u{E012}', // f - formen
    '\u{E014}', // g - anga (ungwe)
    '\u{E01E}', // h - hyarmen
    '\u{E002}', // i - short carrier
    '\u{E01E}', // j - hyarmen (approx)
    '\u{E006}', // k - calma
    '\u{E024}', // l - lambe
    '\u{E022}', // m - malta (vala)
    '\u{E020}', // n - numen
    '\u{E005}', // o - anna variant
    '\u{E008}', // p - parma
    '\u{E00A}', // q - quesse
    '\u{E026}', // r - romen
    '\u{E01C}', // s - silme
    '\u{E004}', // t - tinco
    '\u{E016}', // u - ure
    '\u{E00E}', // v - ampa
    '\u{E018}', // w - vilya
    '\u{E006}', // x - calma (approx)
    '\u{E002}', // y - short carrier (approx)
    '\u{E01A}', // z - esse
];
