use anyhow::{Context, Result};
use elements::ToPrintCommand;
use escpos::{
    driver::{Driver, NetworkDriver, UsbDriver},
    printer::Printer,
    printer_options::PrinterOptions,
    utils::{JustifyMode, Protocol, UnderlineMode},
};
use log::trace;

mod cp437;
pub mod elements;

pub const CPL: u8 = 48; // characters per line
const IP: &str = "192.168.1.87";
const PORT: u16 = 9100;
const VENDOR_ID: u16 = 0x0FE6;
const PRODUCT_ID: u16 = 0x811E;

/// Enum-based printer abstraction for runtime driver selection without dyn.
pub enum AnyPrinter {
    Usb(Printer<UsbDriver>),
    Network(Printer<NetworkDriver>),
}

impl AnyPrinter {
    pub fn feed(&mut self) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.feed()?;
            }
            AnyPrinter::Network(p) => {
                p.feed()?;
            }
        }
        Ok(())
    }

    pub fn print(&mut self) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.print()?;
            }
            AnyPrinter::Network(p) => {
                p.print()?;
            }
        }
        Ok(())
    }

    pub fn print_cut(&mut self) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.print_cut()?;
            }
            AnyPrinter::Network(p) => {
                p.print_cut()?;
            }
        }
        Ok(())
    }

    pub fn write(&mut self, text: &str) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.write(text)?;
            }
            AnyPrinter::Network(p) => {
                p.write(text)?;
            }
        }
        Ok(())
    }

    pub fn justify(&mut self, mode: JustifyMode) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.justify(mode)?;
            }
            AnyPrinter::Network(p) => {
                p.justify(mode)?;
            }
        }
        Ok(())
    }

    pub fn bold(&mut self, enabled: bool) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.bold(enabled)?;
            }
            AnyPrinter::Network(p) => {
                p.bold(enabled)?;
            }
        }
        Ok(())
    }

    pub fn underline(&mut self, mode: UnderlineMode) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.underline(mode)?;
            }
            AnyPrinter::Network(p) => {
                p.underline(mode)?;
            }
        }
        Ok(())
    }

    pub fn size(&mut self, width: u8, height: u8) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.size(width, height)?;
            }
            AnyPrinter::Network(p) => {
                p.size(width, height)?;
            }
        }
        Ok(())
    }

    pub fn reset_size(&mut self) -> Result<()> {
        match self {
            AnyPrinter::Usb(p) => {
                p.reset_size()?;
            }
            AnyPrinter::Network(p) => {
                p.reset_size()?;
            }
        }
        Ok(())
    }
}

#[derive(Default, Debug)]
struct Line {
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
        trace!(
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
    fn add_char(&mut self, sch: elements::StyledChar) -> Option<Line> {
        self.chars.push(sch);
        if self.visual_width() <= CPL as usize {
            return None;
        }
        let remainder = if let Some(wrap_point) = self.find_wrap_point() {
            trace!(
                "Wrapping line at {} for {:?}",
                wrap_point, self.chars[wrap_point]
            );
            let mut remainder = self.chars.split_off(wrap_point);
            if !remainder.is_empty() {
                remainder.remove(0); // Remove whitespace at wrap point
            }
            remainder
        } else {
            trace!("No whitespace found, hard wrap for {:?}", self.chars.last());
            self.chars.split_off(self.chars.len() - 1)
        };

        (!remainder.is_empty()).then(|| Line {
            justify_content: self.justify_content,
            chars: remainder,
        })
    }
}

#[derive(Default)]
pub struct PrintBuilder {
    lines: Vec<Line>,
    cut: bool,
    current_text_size: elements::TextSize,
    current_text_decoration: elements::TextDecoration,
}

impl PrintBuilder {
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
            let current_state = elements::FormatState {
                text_size: self.current_text_size,
                text_decoration: self.current_text_decoration,
            };
            let new_line = current_line.add_char(elements::StyledChar {
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
    pub fn set_justify_content(&mut self, justify: elements::Justify) {
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
    pub fn print_to(&self, printer: &mut AnyPrinter, rows: Option<u32>) -> anyhow::Result<()> {
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
    pub fn print(&self, rows: Option<u32>) -> anyhow::Result<()> {
        let mut printer = establish_usb_printer()?;
        self.print_to(&mut printer, rows)
    }

    /// Print via network connection.
    pub fn network_print(&self, rows: Option<u32>) -> anyhow::Result<()> {
        let mut printer = establish_network_printer()?;
        self.print_to(&mut printer, rows)
    }
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

pub fn establish_network_printer() -> Result<AnyPrinter> {
    let driver = NetworkDriver::open(IP, PORT, None)
        .inspect_err(|_| log::error!("Attempted to connect to {}:{}", IP, PORT))
        .with_context(|| "Failed to open network driver")?;
    Ok(AnyPrinter::Network(build_printer(driver)?))
}

pub fn establish_usb_printer() -> Result<AnyPrinter> {
    let driver = UsbDriver::open(VENDOR_ID, PRODUCT_ID, None, None)
        .inspect_err(|_| log::error!("Attempted to connect to {}:{}", VENDOR_ID, PRODUCT_ID))
        .with_context(|| "Failed to open usb driver")?;
    Ok(AnyPrinter::Usb(build_printer(driver)?))
}
