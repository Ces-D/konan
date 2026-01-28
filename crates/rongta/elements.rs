use anyhow::Result;
use escpos::utils::{JustifyMode, UnderlineMode};

use crate::{cp437, AnyPrinter};

pub trait ToPrintCommand {
    fn to_print_command(&self, printer: &mut AnyPrinter) -> Result<()>;
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum TextSize {
    #[default]
    Medium,
    Large,
    ExtraLarge,
}
impl TextSize {
    /// Returns the visual width of a character with this text size.
    /// Medium = 1 column, Large = 2 columns, ExtraLarge = 3 columns.
    pub fn char_width(&self) -> usize {
        match self {
            TextSize::Medium => 1,
            TextSize::Large => 2,
            TextSize::ExtraLarge => 3,
        }
    }
}

impl ToPrintCommand for TextSize {
    fn to_print_command(&self, printer: &mut AnyPrinter) -> Result<()> {
        match self {
            TextSize::Medium => printer.reset_size()?,
            TextSize::Large => printer.size(2, 2)?,
            TextSize::ExtraLarge => printer.size(3, 3)?,
        };
        Ok(())
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct TextDecoration {
    pub bold: bool,
    pub underline: bool,
    pub italic: bool,
}
impl ToPrintCommand for TextDecoration {
    fn to_print_command(&self, printer: &mut AnyPrinter) -> Result<()> {
        match self.bold {
            true => printer.bold(true)?,
            false => printer.bold(false)?,
        };
        match self.underline {
            true => printer.underline(UnderlineMode::Single)?,
            false => printer.underline(UnderlineMode::None)?,
        };
        match self.italic {
            true => printer.underline(UnderlineMode::Single)?,
            false => printer.underline(UnderlineMode::None)?,
        };
        Ok(())
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Justify {
    #[default]
    Left,
    Center,
    Right,
}
impl ToPrintCommand for Justify {
    fn to_print_command(&self, printer: &mut AnyPrinter) -> Result<()> {
        match self {
            Justify::Left => printer.justify(JustifyMode::LEFT)?,
            Justify::Center => printer.justify(JustifyMode::CENTER)?,
            Justify::Right => printer.justify(JustifyMode::RIGHT)?,
        };
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct FormatState {
    pub text_size: TextSize,
    pub text_decoration: TextDecoration,
}

#[derive(Clone, Debug)]
pub struct StyledChar {
    pub ch: char,
    pub state: FormatState,
}
impl ToPrintCommand for StyledChar {
    fn to_print_command(&self, printer: &mut AnyPrinter) -> Result<()> {
        // Normalize typographic characters to ASCII equivalents before CP437 validation
        let normalized_ch = cp437::normalize_char(self.ch).unwrap_or(self.ch);
        let ascii_content = cp437::cp437_char_only(normalized_ch)?;
        self.state.text_size.to_print_command(printer)?;
        self.state.text_decoration.to_print_command(printer)?;
        printer.write(&ascii_content.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod text_size {
        use super::*;

        #[test]
        fn medium_has_width_1() {
            assert_eq!(TextSize::Medium.char_width(), 1);
        }

        #[test]
        fn large_has_width_2() {
            assert_eq!(TextSize::Large.char_width(), 2);
        }

        #[test]
        fn extra_large_has_width_3() {
            assert_eq!(TextSize::ExtraLarge.char_width(), 3);
        }

        #[test]
        fn default_is_medium() {
            assert_eq!(TextSize::default(), TextSize::Medium);
        }
    }

    mod text_decoration {
        use super::*;

        #[test]
        fn default_has_no_decorations() {
            let decoration = TextDecoration::default();
            assert!(!decoration.bold);
            assert!(!decoration.underline);
            assert!(!decoration.italic);
        }

        #[test]
        fn can_set_bold() {
            let decoration = TextDecoration {
                bold: true,
                ..Default::default()
            };
            assert!(decoration.bold);
            assert!(!decoration.underline);
        }

        #[test]
        fn can_set_multiple_decorations() {
            let decoration = TextDecoration {
                bold: true,
                underline: true,
                italic: false,
            };
            assert!(decoration.bold);
            assert!(decoration.underline);
            assert!(!decoration.italic);
        }
    }

    mod justify {
        use super::*;

        #[test]
        fn default_is_left() {
            assert_eq!(Justify::default(), Justify::Left);
        }

        #[test]
        fn variants_are_distinct() {
            assert_ne!(Justify::Left, Justify::Center);
            assert_ne!(Justify::Center, Justify::Right);
            assert_ne!(Justify::Left, Justify::Right);
        }
    }

    mod format_state {
        use super::*;

        #[test]
        fn default_has_medium_size_and_no_decoration() {
            let state = FormatState::default();
            assert_eq!(state.text_size, TextSize::Medium);
            assert_eq!(state.text_decoration, TextDecoration::default());
        }

        #[test]
        fn can_construct_with_custom_values() {
            let state = FormatState {
                text_size: TextSize::Large,
                text_decoration: TextDecoration {
                    bold: true,
                    underline: false,
                    italic: false,
                },
            };
            assert_eq!(state.text_size, TextSize::Large);
            assert!(state.text_decoration.bold);
        }
    }

    mod styled_char {
        use super::*;

        #[test]
        fn can_construct_with_char_and_state() {
            let styled = StyledChar {
                ch: 'A',
                state: FormatState::default(),
            };
            assert_eq!(styled.ch, 'A');
        }

        #[test]
        fn preserves_format_state() {
            let state = FormatState {
                text_size: TextSize::ExtraLarge,
                text_decoration: TextDecoration {
                    bold: true,
                    underline: true,
                    italic: false,
                },
            };
            let styled = StyledChar { ch: 'X', state };
            assert_eq!(styled.state.text_size, TextSize::ExtraLarge);
            assert!(styled.state.text_decoration.bold);
            assert!(styled.state.text_decoration.underline);
        }

        #[test]
        fn can_clone() {
            let styled = StyledChar {
                ch: 'Z',
                state: FormatState {
                    text_size: TextSize::Large,
                    text_decoration: TextDecoration::default(),
                },
            };
            let cloned = styled.clone();
            assert_eq!(cloned.ch, styled.ch);
            assert_eq!(cloned.state, styled.state);
        }
    }
}
