use anyhow::{Result, bail};
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

#[cfg(test)]
mod tests {
    use super::*;

    mod normalize_char {
        use super::*;

        #[test]
        fn normalizes_left_single_quote() {
            assert_eq!(normalize_char('\u{2018}'), Some('\''));
        }

        #[test]
        fn normalizes_right_single_quote() {
            assert_eq!(normalize_char('\u{2019}'), Some('\''));
        }

        #[test]
        fn normalizes_modifier_letter_apostrophe() {
            assert_eq!(normalize_char('\u{02BC}'), Some('\''));
        }

        #[test]
        fn normalizes_left_double_quote() {
            assert_eq!(normalize_char('\u{201C}'), Some('"'));
        }

        #[test]
        fn normalizes_right_double_quote() {
            assert_eq!(normalize_char('\u{201D}'), Some('"'));
        }

        #[test]
        fn normalizes_en_dash() {
            assert_eq!(normalize_char('\u{2013}'), Some('-'));
        }

        #[test]
        fn normalizes_em_dash() {
            assert_eq!(normalize_char('\u{2014}'), Some('-'));
        }

        #[test]
        fn returns_none_for_regular_ascii() {
            assert_eq!(normalize_char('a'), None);
            assert_eq!(normalize_char('Z'), None);
            assert_eq!(normalize_char('5'), None);
            assert_eq!(normalize_char('-'), None);
            assert_eq!(normalize_char('\''), None);
        }

        #[test]
        fn returns_none_for_cp437_extended() {
            assert_eq!(normalize_char('é'), None);
            assert_eq!(normalize_char('║'), None);
        }
    }

    mod cp437_char_only {
        use super::*;

        #[test]
        fn accepts_ascii_letters() {
            assert!(cp437_char_only('a').is_ok());
            assert!(cp437_char_only('Z').is_ok());
        }

        #[test]
        fn accepts_ascii_digits() {
            assert!(cp437_char_only('0').is_ok());
            assert!(cp437_char_only('9').is_ok());
        }

        #[test]
        fn accepts_ascii_punctuation() {
            assert!(cp437_char_only('.').is_ok());
            assert!(cp437_char_only('!').is_ok());
            assert!(cp437_char_only('@').is_ok());
        }

        #[test]
        fn accepts_space_and_newline() {
            assert!(cp437_char_only(' ').is_ok());
            assert!(cp437_char_only('\n').is_ok());
            assert!(cp437_char_only('\t').is_ok());
        }

        #[test]
        fn accepts_cp437_accented_characters() {
            assert!(cp437_char_only('é').is_ok());
            assert!(cp437_char_only('ü').is_ok());
            assert!(cp437_char_only('ñ').is_ok());
            assert!(cp437_char_only('Ç').is_ok());
        }

        #[test]
        fn accepts_cp437_box_drawing() {
            assert!(cp437_char_only('│').is_ok());
            assert!(cp437_char_only('─').is_ok());
            assert!(cp437_char_only('┌').is_ok());
            assert!(cp437_char_only('╔').is_ok());
            assert!(cp437_char_only('║').is_ok());
        }

        #[test]
        fn accepts_cp437_math_symbols() {
            assert!(cp437_char_only('±').is_ok());
            assert!(cp437_char_only('≥').is_ok());
            assert!(cp437_char_only('≤').is_ok());
            assert!(cp437_char_only('∞').is_ok());
            assert!(cp437_char_only('√').is_ok());
        }

        #[test]
        fn accepts_cp437_currency_symbols() {
            assert!(cp437_char_only('¢').is_ok());
            assert!(cp437_char_only('£').is_ok());
            assert!(cp437_char_only('¥').is_ok());
        }

        #[test]
        fn accepts_cp437_greek_letters() {
            // CP437 includes: α, Γ, π, Σ, σ, µ, τ, Φ, Θ, Ω, δ, φ, ε
            // Note: ß (German sharp S) looks like β but is different
            assert!(cp437_char_only('α').is_ok());
            assert!(cp437_char_only('Γ').is_ok());
            assert!(cp437_char_only('π').is_ok());
            assert!(cp437_char_only('Σ').is_ok());
            assert!(cp437_char_only('Ω').is_ok());
            assert!(cp437_char_only('δ').is_ok());
        }

        #[test]
        fn rejects_emoji() {
            assert!(cp437_char_only('😀').is_err());
            assert!(cp437_char_only('🎉').is_err());
        }

        #[test]
        fn rejects_cjk_characters() {
            assert!(cp437_char_only('中').is_err());
            assert!(cp437_char_only('日').is_err());
        }

        #[test]
        fn rejects_curly_quotes() {
            assert!(cp437_char_only('\u{2019}').is_err()); // right single quote
            assert!(cp437_char_only('\u{201C}').is_err()); // left double quote
        }

        #[test]
        fn rejects_special_unicode() {
            assert!(cp437_char_only('\u{2192}').is_err()); // right arrow →
            assert!(cp437_char_only('\u{2022}').is_err()); // bullet point • (different from CP437's ∙)
        }
    }

    mod is_cp437_char {
        use super::*;

        #[test]
        fn all_cp437_chars_are_valid() {
            for ch in CP437_CHARS {
                assert!(
                    is_cp437_char(ch),
                    "CP437 character '{}' should be valid",
                    ch
                );
            }
        }

        #[test]
        fn all_printable_ascii_are_valid() {
            for code in 0x20u8..=0x7E {
                let ch = code as char;
                assert!(is_cp437_char(ch), "ASCII '{}' should be valid", ch);
            }
        }
    }
}
