use anyhow::{Result, bail};
use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Extended CP437 characters (non-ASCII) for O(1) lookup
static EXTENDED_CP437: LazyLock<HashSet<char>> = LazyLock::new(|| {
    CP437_CHARS
        .iter()
        .copied()
        .filter(|ch| !ch.is_ascii())
        .collect()
});

/// All valid CP437 characters mapped to their Unicode equivalents
pub const CP437_CHARS: [char; 128] = [
    // 0x20-0x2F (standard ASCII)
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ',
    'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ',
    'ª', 'º', '¿', '⌐', '¬', '½', '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕',
    '╣', '║', '╗', '╝', '╜', '╛', '┐', '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦',
    '╠', '═', '╬', '╧', '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐',
    '▀', 'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩', '≡', '±',
    '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', '\u{00A0}', 
];

/// Normalize a single Unicode typographic character to its ASCII equivalent.
/// Returns the ASCII equivalent if applicable, otherwise returns None.
pub fn normalize_char(ch: char) -> Option<char> {
    match ch {
        // Curly apostrophes → straight apostrophe
        '\u{2018}' | '\u{2019}' | '\u{02BC}' => Some('\''),
        // Curly double quotes → straight double quote
        '\u{201C}' | '\u{201D}' => Some('"'),
        // En-dash, em-dash → hyphen-minus
        '\u{2013}' | '\u{2014}' => Some('-'),
        _ => None,
    }
}

/// Normalize a string by converting Unicode typographic characters to ASCII equivalents.
/// - Curly apostrophes (' ') → straight apostrophe (')
/// - Curly quotes (" ") → straight quote (")
/// - En-dash (–), em-dash (—) → hyphen-minus (-)
/// - Ellipsis (…) → three periods (...)
pub fn normalize_to_ascii(s: &str) -> Cow<'_, str> {
    // Fast path: check if any normalization is needed
    let needs_normalization = s.chars().any(|ch| {
        matches!(
            ch,
            '\u{2018}'
                | '\u{2019}'
                | '\u{02BC}'
                | '\u{201C}'
                | '\u{201D}'
                | '\u{2013}'
                | '\u{2014}'
                | '\u{2026}'
        )
    });

    if !needs_normalization {
        return Cow::Borrowed(s);
    }

    let normalized: String = s
        .chars()
        .flat_map(|ch| {
            if ch == '\u{2026}' {
                // Ellipsis → three periods (expands to multiple chars)
                vec!['.', '.', '.']
            } else if let Some(replacement) = normalize_char(ch) {
                vec![replacement]
            } else {
                vec![ch]
            }
        })
        .collect();

    Cow::Owned(normalized)
}

/// Check if a character is valid in CP437.
/// Uses a fast path for ASCII characters and HashSet lookup for extended characters.
fn is_cp437_char(ch: char) -> bool {
    if ch.is_ascii() {
        return true;
    }
    EXTENDED_CP437.contains(&ch)
}

/// Validate that a single character is valid in CP437.
/// Returns the character if valid, or an error.
pub fn cp437_char_only(ch: char) -> Result<char> {
    if is_cp437_char(ch) {
        Ok(ch)
    } else {
        bail!("Non-CP437 character: '{}'", ch)
    }
}
