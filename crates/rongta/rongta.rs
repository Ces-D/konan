use crate::elements::{FormatState, Justify, TextSize};
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

#[derive(Default)]
pub struct RongtaPrinter {
    lines: Vec<line::Line>,
    cut: bool,
    format_state: FormatState,
}

impl RongtaPrinter {
    pub fn new(cut: bool) -> Self {
        Self {
            cut,
            ..Default::default()
        }
    }

    /// Add content to the current line. The content is formatted according to the current formatting state.
    /// This is a more efficient way to add content that needs the same formatting.
    /// Highly recommended to call `new_line()` after adding content to the current line.
    pub fn add_content(&mut self, content: &str) -> Result<()> {
        if self.lines.is_empty() {
            self.lines.push(line::Line::default());
        }
        for char in content.chars() {
            let new_line = {
                let current_line = self
                    .lines
                    .last_mut()
                    .expect("New line should have been added");
                current_line.add_char(elements::StyledChar {
                    ch: char,
                    state: self.format_state,
                })
            };

            if let Some(new_line) = new_line {
                self.lines.push(new_line);
            }
        }
        Ok(())
    }

    pub fn new_line(&mut self) {
        self.lines.push(line::Line::default());
    }

    /// Set the justify content of the last line or add a new line with the given justify content
    pub fn set_justify_content(&mut self, justify: elements::Justify) {
        if let Some(line) = self.lines.last_mut() {
            line.justify_content = justify;
        } else {
            self.lines.push(line::Line::new(Vec::default(), justify));
        }
    }

    /// Set the text size of the next characters
    pub fn set_text_size(&mut self, size: elements::TextSize) {
        self.format_state.text_size = size;
    }

    /// Set the bold state for the next characters
    pub fn set_is_bold(&mut self, bold: bool) {
        self.format_state.is_bold = bold;
    }

    /// Reset all styles for the next characters
    /// If you want to reset the justification you should explicitly set or call `new_line`
    pub fn reset_styles(&mut self) {
        self.format_state = Default::default();
    }

    /// Core printing logic - works with any printer variant.
    pub fn print_to(
        &self,
        printer: &mut printer::AnyPrinter,
        rows: Option<u32>,
    ) -> anyhow::Result<()> {
        let mut last_justify_content = Justify::default();
        let mut last_format_state = FormatState::default();
        if let Some(rows_per_page) = rows {
            let mut line_count = 0;
            for line in &self.lines {
                print_line(
                    line,
                    printer,
                    &mut last_justify_content,
                    &mut last_format_state,
                )?;
                line_count += 1;
                if line_count >= rows_per_page {
                    printer.print_cut()?;
                    line_count = 0;
                }
            }
            if line_count > 0 {
                while line_count < rows_per_page {
                    printer.feed()?;
                    line_count += 1;
                }
                printer.print_cut()?;
            }
        } else {
            for line in &self.lines {
                print_line(
                    line,
                    printer,
                    &mut last_justify_content,
                    &mut last_format_state,
                )?;
            }
            match self.cut {
                true => printer.print_cut()?,
                false => printer.print()?,
            };
        }
        Ok(())
    }

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

fn print_line(
    line: &line::Line,
    printer: &mut printer::AnyPrinter,
    last_justify_content: &mut Justify,
    last_format_state: &mut FormatState,
) -> anyhow::Result<()> {
    if *last_justify_content != line.justify_content {
        line.justify_content.to_print_command(printer)?;
        *last_justify_content = line.justify_content;
    }
    // Some thermal printers ignore GS ! (text size reset) when it follows a
    // sequence of feeds that were issued while a larger size was active. Reset
    // the format state before feeding an empty line so the printer does not
    // carry the previous text size into subsequent lines.
    if line.chars.is_empty() && last_format_state.text_size != TextSize::default() {
        let default = FormatState::default();
        default.to_print_command(printer)?;
        *last_format_state = default;
    }
    for styled_char in &line.chars {
        if *last_format_state != styled_char.state {
            styled_char.state.to_print_command(printer)?;
            *last_format_state = styled_char.state;
        }
        styled_char.to_print_command(printer)?;
    }
    printer.feed()
}
