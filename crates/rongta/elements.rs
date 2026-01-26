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
