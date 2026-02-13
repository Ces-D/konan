use crate::{CPL, elements};

#[derive(Default, Debug)]
pub struct Line {
    pub chars: Vec<elements::StyledChar>,
    pub justify_content: elements::Justify,
}
impl Line {
    /// Calculate the visual width of the line, accounting for text size.
    fn visual_width(&self) -> usize {
        self.chars
            .iter()
            .map(|sc| sc.state.text_size.char_width())
            .sum()
    }

    /// Find the character index where we should soft-wrap (at whitespace).
    /// Returns None if the line fits within CPL or no whitespace is found.
    fn find_wrap_point(&self) -> Option<usize> {
        if self.visual_width() <= CPL as usize {
            return None;
        }
        log::trace!(
            "Finding wrap point for {:?}",
            self.chars.iter().map(|sc| sc.ch).collect::<Vec<char>>()
        );

        // Find the last whitespace before we exceed CPL visual width
        let mut width = 0;
        let mut last_whitespace_idx: Option<usize> = None;

        for (i, sc) in self.chars.iter().enumerate() {
            if sc.ch.is_whitespace() && width <= CPL as usize {
                last_whitespace_idx = Some(i);
            }

            width += sc.state.text_size.char_width();

            // Once we've exceeded CPL, stop looking
            if width > CPL as usize {
                break;
            }
        }

        last_whitespace_idx
    }

    /// Add a character to the line, and return a new line if the line is full.
    /// Uses visual width (accounting for text size) to determine when to wrap.
    pub fn add_char(&mut self, sch: elements::StyledChar) -> Option<Line> {
        self.chars.push(sch);
        if self.visual_width() <= CPL as usize {
            return None;
        }
        let remainder = if let Some(wrap_point) = self.find_wrap_point() {
            log::trace!(
                "Wrapping line at {} for {:?}",
                wrap_point,
                self.chars[wrap_point]
            );
            let mut remainder = self.chars.split_off(wrap_point);
            if !remainder.is_empty() {
                remainder.remove(0); // Remove whitespace at wrap point
            }
            remainder
        } else {
            log::trace!("No whitespace found, hard wrap for {:?}", self.chars.last());
            self.chars.split_off(self.chars.len() - 1)
        };

        (!remainder.is_empty()).then(|| Line {
            justify_content: self.justify_content,
            chars: remainder,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::elements::{FormatState, StyledChar, TextSize};

    use super::*;
    fn styled_char(ch: char) -> StyledChar {
        StyledChar {
            ch,
            state: FormatState::default(),
        }
    }

    fn styled_char_large(ch: char) -> StyledChar {
        StyledChar {
            ch,
            state: FormatState {
                text_size: TextSize::Large,
                is_bold: false,
            },
        }
    }

    fn styled_char_extra_large(ch: char) -> StyledChar {
        StyledChar {
            ch,
            state: FormatState {
                text_size: TextSize::ExtraLarge,
                is_bold: false,
            },
        }
    }

    mod visual_width {
        use super::*;

        #[test]
        fn empty_line_has_zero_width() {
            let line = Line::default();
            assert_eq!(line.visual_width(), 0);
        }

        #[test]
        fn medium_chars_count_as_one() {
            let mut line = Line::default();
            line.chars.push(styled_char('a'));
            line.chars.push(styled_char('b'));
            line.chars.push(styled_char('c'));
            assert_eq!(line.visual_width(), 3);
        }

        #[test]
        fn large_chars_count_as_two() {
            let mut line = Line::default();
            line.chars.push(styled_char_large('a'));
            line.chars.push(styled_char_large('b'));
            assert_eq!(line.visual_width(), 4);
        }

        #[test]
        fn extra_large_chars_count_as_three() {
            let mut line = Line::default();
            line.chars.push(styled_char_extra_large('a'));
            line.chars.push(styled_char_extra_large('b'));
            assert_eq!(line.visual_width(), 6);
        }

        #[test]
        fn mixed_sizes_sum_correctly() {
            let mut line = Line::default();
            line.chars.push(styled_char('a')); // 1
            line.chars.push(styled_char_large('b')); // 2
            line.chars.push(styled_char_extra_large('c')); // 3
            assert_eq!(line.visual_width(), 6);
        }
    }

    mod find_wrap_point {
        use super::*;

        #[test]
        fn returns_none_when_line_fits() {
            let mut line = Line::default();
            for ch in "Hello World".chars() {
                line.chars.push(styled_char(ch));
            }
            assert!(line.find_wrap_point().is_none());
        }

        #[test]
        fn finds_last_whitespace_before_cpl() {
            let mut line = Line::default();
            // Create a line exactly at CPL with a space in the middle
            // "Hello World" repeated to exceed CPL (48)
            let text = "Hello World Hello World Hello World Hello World X";
            for ch in text.chars() {
                line.chars.push(styled_char(ch));
            }
            // Should find a wrap point at one of the spaces
            let wrap = line.find_wrap_point();
            assert!(wrap.is_some());
            // Wrap point should be at a space
            let idx = wrap.unwrap();
            assert!(line.chars[idx].ch.is_whitespace());
        }

        #[test]
        fn returns_none_for_no_whitespace_in_short_line() {
            let mut line = Line::default();
            for ch in "NoSpaces".chars() {
                line.chars.push(styled_char(ch));
            }
            assert!(line.find_wrap_point().is_none());
        }
    }

    mod add_char {
        use crate::elements::Justify;

        use super::*;

        #[test]
        fn returns_none_when_line_not_full() {
            let mut line = Line::default();
            let result = line.add_char(styled_char('a'));
            assert!(result.is_none());
            assert_eq!(line.chars.len(), 1);
        }

        #[test]
        fn returns_none_until_cpl_exceeded() {
            let mut line = Line::default();
            for _ in 0..CPL {
                let result = line.add_char(styled_char('a'));
                assert!(result.is_none());
            }
            assert_eq!(line.visual_width(), CPL as usize);
        }

        #[test]
        fn returns_new_line_when_cpl_exceeded() {
            let mut line = Line::default();
            // Fill exactly to CPL
            for _ in 0..CPL {
                line.add_char(styled_char('a'));
            }
            // Adding one more should trigger wrap
            let result = line.add_char(styled_char('b'));
            assert!(result.is_some());
        }

        #[test]
        fn soft_wraps_at_whitespace() {
            let mut line = Line::default();
            // Add "word " pattern that will exceed CPL
            let text = "word word word word word word word word word word!";
            for ch in text.chars() {
                if let Some(new_line) = line.add_char(styled_char(ch)) {
                    // The new line should start with "word" (after space removed)
                    assert!(!new_line.chars.is_empty(), "New line should have content");
                    // The original line should end without trailing space
                    if let Some(last) = line.chars.last() {
                        // After wrap, the space should be removed
                        assert!(
                            !last.ch.is_whitespace() || line.visual_width() <= CPL as usize,
                            "Line should wrap properly"
                        );
                    }
                    break;
                }
            }
        }

        #[test]
        fn hard_wraps_when_no_whitespace() {
            let mut line = Line::default();
            // Add a string with no whitespace that exceeds CPL
            for _ in 0..=CPL {
                line.add_char(styled_char('x'));
            }
            // The line should have wrapped
            assert!(
                line.visual_width() <= CPL as usize,
                "Line should be within CPL after hard wrap"
            );
        }

        #[test]
        fn preserves_justify_content_on_wrap() {
            let mut line = Line {
                justify_content: Justify::Center,
                ..Default::default()
            };
            // Fill beyond CPL
            for _ in 0..=CPL {
                if let Some(new_line) = line.add_char(styled_char('a')) {
                    assert_eq!(
                        new_line.justify_content,
                        Justify::Center,
                        "Wrapped line should preserve justify_content"
                    );
                    return;
                }
            }
            panic!("Expected line to wrap");
        }

        #[test]
        fn large_chars_trigger_earlier_wrap() {
            let mut line = Line::default();
            // Large chars take 2 columns, so we need CPL/2 chars
            let chars_needed = (CPL as usize / 2) + 1;
            let mut wrapped = false;
            for _ in 0..chars_needed {
                if line.add_char(styled_char_large('W')).is_some() {
                    wrapped = true;
                    break;
                }
            }
            assert!(wrapped, "Large chars should wrap earlier");
        }
    }
}
