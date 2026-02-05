use anyhow::{Context, Result};
use elements::ToPrintCommand;
use escpos::{
    driver::{ConsoleDriver, Driver, NetworkDriver, UsbDriver},
    printer::Printer,
    printer_options::PrinterOptions,
    utils::Protocol,
};

mod cp437;
pub mod elements;
mod line;
mod printer;

pub const CPL: u8 = 48; // characters per line

/// Call the RongtaPrinter in a consistent way
pub trait ToBuilderCommand {
    fn to_builder_command(&self, builder: &mut RongtaPrinter) -> Result<()>;
}

#[derive(Default)]
pub struct RongtaPrinter {
    lines: Vec<line::Line>,
    cut: bool,
    current_text_size: elements::TextSize,
    current_text_decoration: elements::TextDecoration,
}

impl RongtaPrinter {
    pub fn new(cut: bool) -> Self {
        Self {
            cut,
            ..Default::default()
        }
    }

    fn current_line_justify_content(&self) -> elements::Justify {
        if self.lines.is_empty() {
            Default::default()
        } else {
            self.lines.last().unwrap().justify_content
        }
    }

    /// Add a character to the current line. Provides greater control over formatting.
    pub fn add_char_content(&mut self, content: elements::StyledChar) -> Result<()> {
        let mut current_line = self.lines.pop().unwrap_or_else(|| line::Line {
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
        if self.lines.is_empty() {
            self.lines.push(line::Line {
                justify_content: self.current_line_justify_content(),
                ..Default::default()
            });
        }

        for char in content.chars() {
            let current_state = elements::FormatState {
                text_size: self.current_text_size,
                text_decoration: self.current_text_decoration,
            };
            let new_line = {
                let current_line = self.lines.last_mut().unwrap();
                current_line.add_char(elements::StyledChar {
                    ch: char,
                    state: current_state,
                })
            };

            if let Some(new_line) = new_line {
                self.lines.push(new_line);
            }
        }

        Ok(())
    }

    pub fn new_line(&mut self) {
        self.lines.push(line::Line {
            justify_content: self.current_line_justify_content(),
            ..Default::default()
        });
    }

    /// Set the justify content of the last line or add a new line with the given justify content
    pub fn set_justify_content(&mut self, justify: elements::Justify) {
        if let Some(line) = self.lines.last_mut() {
            line.justify_content = justify;
        } else {
            self.lines.push(line::Line {
                justify_content: justify,
                ..Default::default()
            });
        }
    }

    /// Set the text size of the next characters
    pub fn set_text_size(&mut self, size: elements::TextSize) {
        self.current_text_size = size;
    }

    /// Set the text decoration of the next characters
    pub fn set_text_decoration(&mut self, decoration: elements::TextDecoration) {
        self.current_text_decoration = decoration;
    }

    pub fn reset_styles(&mut self) {
        self.current_text_size = elements::TextSize::default();
        self.current_text_decoration = elements::TextDecoration::default();
        self.set_justify_content(elements::Justify::Left);
    }

    /// Core printing logic - works with any printer variant.
    pub fn print_to(
        &self,
        printer: &mut printer::AnyPrinter,
        rows: Option<u32>,
    ) -> anyhow::Result<()> {
        if let Some(rows_per_page) = rows {
            // Paginated printing with cuts after each page
            let mut line_count = 0;
            for line in &self.lines {
                line.justify_content.to_print_command(printer)?;
                for styled_char in &line.chars {
                    styled_char.to_print_command(printer)?;
                }
                printer.feed()?;
                line_count += 1;
                if line_count >= rows_per_page {
                    printer.print_cut()?;
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
            for line in &self.lines {
                line.justify_content.to_print_command(printer)?;
                for styled_char in &line.chars {
                    styled_char.to_print_command(printer)?;
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

    /// Print via USB connection.
    pub fn print(&self, rows: Option<u32>, driver: SupportedDriver) -> Result<()> {
        let mut printer = match driver {
            SupportedDriver::Console => {
                let driver = ConsoleDriver::open(true);
                printer::AnyPrinter::Console(build_printer(driver)?)
            }
            SupportedDriver::Usb(vendor_id, product_id) => {
                let driver = UsbDriver::open(vendor_id, product_id, None, None)
                    .inspect_err(|_| {
                        log::error!("Attempted to connect to {}:{}", vendor_id, product_id)
                    })
                    .with_context(|| "Failed to open usb driver")?;
                printer::AnyPrinter::Usb(build_printer(driver)?)
            }
            SupportedDriver::Network(host, port) => {
                let driver = NetworkDriver::open(&host, port, None)
                    .inspect_err(|_| log::error!("Attempted to connect to {}:{}", host, port))
                    .with_context(|| "Failed to open network driver")?;
                printer::AnyPrinter::Network(build_printer(driver)?)
            }
        };
        self.print_to(&mut printer, rows)
    }
}

#[derive(Clone)]
pub enum SupportedDriver {
    Console,
    Usb(u16, u16),
    Network(String, u16),
}

fn build_printer<D>(driver: D) -> Result<Printer<D>>
where
    D: Driver,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use elements::{FormatState, Justify, StyledChar, TextDecoration, TextSize};

    mod print_builder {
        use super::*;

        #[test]
        fn new_creates_empty_builder() {
            let builder = RongtaPrinter::new(true);
            assert!(builder.lines.is_empty());
        }

        #[test]
        fn new_sets_cut_flag() {
            let builder = RongtaPrinter::new(true);
            assert!(builder.cut);

            let builder = RongtaPrinter::new(false);
            assert!(!builder.cut);
        }

        #[test]
        fn add_content_creates_line() {
            let mut builder = RongtaPrinter::new(false);
            builder.add_content("Hello").unwrap();
            assert_eq!(builder.lines.len(), 1);
        }

        #[test]
        fn add_content_adds_chars() {
            let mut builder = RongtaPrinter::new(false);
            builder.add_content("Hi").unwrap();
            assert_eq!(builder.lines[0].chars.len(), 2);
            assert_eq!(builder.lines[0].chars[0].ch, 'H');
            assert_eq!(builder.lines[0].chars[1].ch, 'i');
        }

        #[test]
        fn new_line_adds_empty_line() {
            let mut builder = RongtaPrinter::new(false);
            builder.add_content("First").unwrap();
            builder.new_line();
            builder.add_content("Second").unwrap();
            assert_eq!(builder.lines.len(), 2);
        }

        #[test]
        fn set_justify_content_affects_current_line() {
            let mut builder = RongtaPrinter::new(false);
            builder.add_content("Text").unwrap();
            builder.set_justify_content(Justify::Center);
            assert_eq!(builder.lines[0].justify_content, Justify::Center);
        }

        #[test]
        fn set_justify_content_creates_line_if_empty() {
            let mut builder = RongtaPrinter::new(false);
            builder.set_justify_content(Justify::Right);
            assert_eq!(builder.lines.len(), 1);
            assert_eq!(builder.lines[0].justify_content, Justify::Right);
        }

        #[test]
        fn set_text_size_affects_subsequent_content() {
            let mut builder = RongtaPrinter::new(false);
            builder.set_text_size(TextSize::Large);
            builder.add_content("Big").unwrap();
            assert_eq!(builder.lines[0].chars[0].state.text_size, TextSize::Large);
        }

        #[test]
        fn set_text_decoration_affects_subsequent_content() {
            let mut builder = RongtaPrinter::new(false);
            builder.set_text_decoration(TextDecoration {
                bold: true,
                underline: false,
                italic: false,
            });
            builder.add_content("Bold").unwrap();
            assert!(builder.lines[0].chars[0].state.text_decoration.bold);
        }

        #[test]
        fn reset_styles_clears_formatting() {
            let mut builder = RongtaPrinter::new(false);
            builder.set_text_size(TextSize::ExtraLarge);
            builder.set_text_decoration(TextDecoration {
                bold: true,
                underline: true,
                italic: true,
            });
            builder.set_justify_content(Justify::Right);
            builder.reset_styles();
            builder.add_content("Normal").unwrap();

            let last_line = builder.lines.last().unwrap();
            assert_eq!(last_line.justify_content, Justify::Left);
            assert_eq!(last_line.chars[0].state.text_size, TextSize::Medium);
            assert!(!last_line.chars[0].state.text_decoration.bold);
        }

        #[test]
        fn mixed_formatting_within_line() {
            let mut builder = RongtaPrinter::new(false);
            builder.add_content("Normal ").unwrap();
            builder.set_text_decoration(TextDecoration {
                bold: true,
                underline: false,
                italic: false,
            });
            builder.add_content("Bold").unwrap();

            let line = &builder.lines[0];
            // First chars should not be bold
            assert!(!line.chars[0].state.text_decoration.bold);
            // Last chars should be bold (after "Normal ")
            assert!(line.chars[7].state.text_decoration.bold);
        }

        #[test]
        fn new_line_inherits_justify_from_previous() {
            let mut builder = RongtaPrinter::new(false);
            builder.set_justify_content(Justify::Center);
            builder.add_content("Line 1").unwrap();
            builder.new_line();
            builder.add_content("Line 2").unwrap();

            assert_eq!(builder.lines[0].justify_content, Justify::Center);
            assert_eq!(builder.lines[1].justify_content, Justify::Center);
        }

        #[test]
        fn auto_wraps_long_content() {
            let mut builder = RongtaPrinter::new(false);
            // Add content longer than CPL
            let long_text = "a".repeat(CPL as usize + 10);
            builder.add_content(&long_text).unwrap();

            assert!(
                builder.lines.len() >= 2,
                "Long content should wrap to multiple lines"
            );
        }

        #[test]
        fn add_char_content_allows_fine_control() {
            let mut builder = RongtaPrinter::new(false);
            let styled = StyledChar {
                ch: 'X',
                state: FormatState {
                    text_size: TextSize::Large,
                    text_decoration: TextDecoration {
                        bold: true,
                        underline: false,
                        italic: false,
                    },
                },
            };
            builder.add_char_content(styled.clone()).unwrap();
            assert_eq!(builder.lines[0].chars[0].ch, 'X');
            assert_eq!(builder.lines[0].chars[0].state.text_size, TextSize::Large);
        }
    }

    mod cpl_constant {
        use super::*;

        #[test]
        fn cpl_is_48() {
            assert_eq!(CPL, 48);
        }
    }
}
