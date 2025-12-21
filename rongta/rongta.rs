use anyhow::{Result, anyhow, bail};
use ascii::AsciiString;
use escpos::{
    driver::NetworkDriver,
    printer::Printer,
    printer_options::PrinterOptions,
    utils::{Protocol, UnderlineMode},
};
use log::{error, trace};
use std::str::FromStr;

pub const CPL: u8 = 48; // characters per line
const IP: &str = "192.168.1.87";
const PORT: u16 = 9100;

trait ToPrintCommand {
    fn to_print_command(&self, printer: &mut Printer<NetworkDriver>) -> Result<()>;
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum TextSize {
    #[default]
    Medium,
    Large,
    ExtraLarge,
}
impl ToPrintCommand for TextSize {
    fn to_print_command(&self, printer: &mut Printer<NetworkDriver>) -> Result<()> {
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
    fn to_print_command(&self, printer: &mut Printer<NetworkDriver>) -> Result<()> {
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
    fn to_print_command(&self, printer: &mut Printer<NetworkDriver>) -> Result<()> {
        match self {
            Justify::Left => printer.justify(escpos::utils::JustifyMode::LEFT)?,
            Justify::Center => printer.justify(escpos::utils::JustifyMode::CENTER)?,
            Justify::Right => printer.justify(escpos::utils::JustifyMode::RIGHT)?,
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
    fn to_print_command(&self, printer: &mut Printer<NetworkDriver>) -> Result<()> {
        let ascii_content = ascii_only(&self.ch.to_string())?;
        self.state.text_size.to_print_command(printer)?;
        self.state.text_decoration.to_print_command(printer)?;
        printer.write(&ascii_content)?;
        Ok(())
    }
}

#[derive(Default, Debug)]
struct Line {
    pub chars: Vec<StyledChar>,
    pub justify_content: Justify,
}
impl Line {
    fn find_wrap_point(&self) -> Option<usize> {
        if self.chars.len() <= CPL as usize {
            return None;
        }
        trace!(
            "Finding wrap point for {:?}",
            self.chars.iter().map(|sc| sc.ch).collect::<Vec<char>>()
        );
        self.chars
            .iter()
            .take(CPL as usize)
            .enumerate()
            .rfind(|(_, sc)| sc.ch.is_whitespace())
            .map(|(idx, _)| idx)
    }

    /// Add a character to the line, and return a new line if the line is full
    fn add_char(&mut self, sch: StyledChar) -> Option<Line> {
        self.chars.push(sch);
        if self.chars.len() > CPL as usize {
            if let Some(wrap_point) = self.find_wrap_point() {
                trace!(
                    "Wrapping line at {} for {:?}",
                    wrap_point, self.chars[wrap_point]
                );
                let mut remainder = self.chars.split_off(wrap_point);
                // Remove the whitespace character at the wrap point
                if !remainder.is_empty() {
                    remainder.remove(0);
                }
                if remainder.is_empty() {
                    return None;
                } else {
                    let new_line = Line {
                        justify_content: self.justify_content,
                        chars: remainder,
                    };
                    return Some(new_line);
                }
            } else {
                trace!("No whitespace found, hard wrap for {:?}", self.chars.last());
                // No whitespace found, hard wrap
                let remainder = self.chars.split_off(CPL as usize);
                if remainder.is_empty() {
                    return None;
                } else {
                    let new_line = Line {
                        justify_content: self.justify_content,
                        chars: remainder,
                    };
                    return Some(new_line);
                }
            }
        }

        None
    }
}

#[derive(Default)]
pub struct PrintBuilder {
    lines: Vec<Line>,
    cut: bool,
    current_text_size: TextSize,
    current_text_decoration: TextDecoration,
}

impl PrintBuilder {
    pub fn new(cut: bool) -> Self {
        Self {
            cut,
            ..Default::default()
        }
    }

    fn current_line_justify_content(&self) -> Justify {
        if self.lines.is_empty() {
            Default::default()
        } else {
            self.lines.last().unwrap().justify_content
        }
    }

    /// Add a character to the current line. Provides greater control over formatting.
    pub fn add_char_content(&mut self, content: StyledChar) -> Result<()> {
        let mut current_line = self.lines.pop().unwrap_or_else(|| Line {
            justify_content: self.current_line_justify_content(),
            ..Default::default()
        });
        let new_line = current_line.add_char(content);
        self.lines.push(current_line);
        if let Some(new_line) = new_line {
            self.lines.push(new_line);
        }
        Ok(())
    }

    /// Add content to the current line. The content is formatted according to the current formatting state.
    /// This is a more efficient way to add content that needs the same formatting.
    /// Highly recommended to call `new_line()` after adding content to the current line.
    pub fn add_content(&mut self, content: &str) -> Result<()> {
        let mut current_line = self.lines.pop().unwrap_or_else(|| Line {
            justify_content: self.current_line_justify_content(),
            ..Default::default()
        });

        for char in content.chars() {
            let current_state = FormatState {
                text_size: self.current_text_size,
                text_decoration: self.current_text_decoration,
            };
            let new_line = current_line.add_char(StyledChar {
                ch: char,
                state: current_state,
            });

            if let Some(new_line) = new_line {
                self.lines.push(current_line);
                current_line = new_line;
            }
        }

        self.lines.push(current_line);
        Ok(())
    }

    pub fn new_line(&mut self) {
        self.lines.push(Line {
            justify_content: self.current_line_justify_content(),
            ..Default::default()
        });
    }

    /// Set the justify content of the last line or add a new line with the given justify content
    pub fn set_justify_content(&mut self, justify: Justify) {
        if let Some(line) = self.lines.last_mut() {
            line.justify_content = justify;
        } else {
            self.lines.push(Line {
                justify_content: justify,
                ..Default::default()
            });
        }
    }

    /// Set the text size of the next characters
    pub fn set_text_size(&mut self, size: TextSize) {
        self.current_text_size = size;
    }

    /// Set the text decoration of the next characters
    pub fn set_text_decoration(&mut self, decoration: TextDecoration) {
        self.current_text_decoration = decoration;
    }

    pub fn print(&self, rows: Option<u32>) -> Result<()> {
        if let Some(rows_per_page) = rows {
            // Paginated printing with cuts after each page
            let mut line_count = 0;
            let mut printer = establish_rongta_printer()?;
            for line in &self.lines {
                line.justify_content.to_print_command(&mut printer)?;
                for styled_char in &line.chars {
                    styled_char.to_print_command(&mut printer)?;
                }
                printer.feed()?;
                line_count += 1;
                if line_count >= rows_per_page {
                    printer.print_cut()?;
                    // printer = establish_rongta_printer()?; #TODO: if the app continues to work,
                    // delete this comment
                    line_count = 0;
                }
            }

            // Pad remaining lines to fill the page
            if line_count > 0 {
                while line_count < rows_per_page {
                    printer.feed()?;
                    line_count += 1;
                }
                printer.print_cut()?;
            }
        } else {
            // Original behavior
            let mut printer = establish_rongta_printer()?;
            for line in &self.lines {
                line.justify_content.to_print_command(&mut printer)?;
                for styled_char in &line.chars {
                    styled_char.to_print_command(&mut printer)?;
                }
                printer.feed()?;
            }
            match self.cut {
                true => printer.print_cut()?,
                false => printer.print()?,
            };
        }
        Ok(())
    }
}

fn ascii_only(s: &str) -> Result<String> {
    match AsciiString::from_str(s) {
        Ok(s) => Ok(s.into()),
        Err(e) => bail!(
            "Non-ASCII characters detected in '{}': {}",
            s,
            s.chars().nth(e.valid_up_to()).unwrap()
        ),
    }
}

pub fn establish_rongta_printer() -> Result<Printer<NetworkDriver>> {
    // 1) Open network driver
    let driver = match NetworkDriver::open(IP, PORT, None) {
        Ok(driver) => Ok(driver),
        Err(e) => {
            error!("Error opening network driver: {:?}", e);
            Err(anyhow!("Failed to open {}:{}", IP, PORT))
        }
    }?;

    // 2) Build printer
    let mut printer = Printer::new(
        driver,
        Protocol::default(),
        Some(PrinterOptions::new(
            Some(escpos::utils::PageCode::PC437),
            None,
            // Some(DebugMode::Dec), // set to None to disable debug
            CPL,
        )),
    );
    printer.flip(false)?;
    printer.reset()?;

    Ok(printer)
}
