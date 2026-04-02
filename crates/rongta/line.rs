use crate::{CPL, elements};

#[derive(Default, Debug)]
pub struct Line {
    pub chars: Vec<elements::StyledChar>,
    pub justify_content: elements::Justify,
    cached_width: usize,
}
impl Line {
    pub fn new(chars: Vec<elements::StyledChar>, justify_content: elements::Justify) -> Self {
        let cached_width = chars.iter().map(|sc| sc.state.text_size.char_width()).sum();
        Self {
            chars,
            justify_content,
            cached_width,
        }
    }
    /// Find the character index where we should soft-wrap (at whitespace).
    /// Returns None if the line fits within CPL or no whitespace is found.
    fn find_wrap_point(&self) -> Option<usize> {
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
        let char_width = sch.state.text_size.char_width();
        self.cached_width += char_width;
        self.chars.push(sch);
        if self.cached_width <= CPL as usize {
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

        (!remainder.is_empty()).then_some(Line::new(remainder, self.justify_content))
    }
}
