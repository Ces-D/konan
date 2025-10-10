use anyhow::{Result, anyhow, bail};
use ascii::AsciiString;
use escpos::{
    driver::NetworkDriver,
    printer::Printer,
    printer_options::PrinterOptions,
    utils::{DebugMode, Protocol, UnderlineMode},
};
use log::{error, info};
use std::str::FromStr;

const CPL: u8 = 48; // characters per line
const IP: &str = "192.168.1.87";
const PORT: u16 = 9100;

#[derive(Default, Clone, Copy)]
pub enum TextSize {
    #[default]
    Medium,
    Large,
    ExtraLarge,
}

pub struct Word {
    content: String,
    text_size: TextSize,
    is_bold: bool,
    is_underlined: bool,
}

#[derive(Default)]
struct Line {
    words: Vec<Word>,
    character_len: usize,
}

impl Line {
    pub fn push(&mut self, word: Word) {
        info!("Pushing {} to line", word.content);
        self.character_len += word.content.len();
        self.words.push(word);
    }
    pub fn full_len(&self) -> usize {
        if self.words.len() == 0 {
            0
        } else {
            self.character_len + (self.words.len() - 1) // for space between words
        }
    }
}

#[derive(Default)]
pub struct PrintBuilder {
    content: Vec<Line>,
    pub cut: bool,
}

impl PrintBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_content(
        &mut self,
        content: &str,
        text_size: TextSize,
        is_bold: bool,
        is_underlined: bool,
    ) -> Result<()> {
        let mut line = Line::default();
        for word in content.split_ascii_whitespace() {
            info!("Attempting to add word: {}", word);
            if line.full_len() + word.len() + 1 >= CPL as usize {
                self.content.push(line);
                line = Line::default();
            }
            line.push(Word {
                content: ascii_only(word)?,
                text_size,
                is_bold,
                is_underlined,
            });
        }
        self.content.push(line);
        Ok(())
    }

    // Prints added content in a formatted way.
    pub fn print(&self, mut printer: Printer<NetworkDriver>) -> Result<()> {
        for line in self.content.iter() {
            for word in line.words.iter() {
                printer
                    .bold(word.is_bold)?
                    .underline(match word.is_underlined {
                        true => UnderlineMode::Single,
                        false => UnderlineMode::None,
                    })?;
                match word.text_size {
                    TextSize::Medium => printer.reset_size()?,
                    TextSize::Large => printer.size(2, 2)?,
                    TextSize::ExtraLarge => printer.size(3, 3)?,
                };
                printer.write(&word.content)?;
                printer.write(" ")?;
            }
            printer.feed()?;
        }
        if self.cut {
            printer.print_cut()?;
        } else {
            printer.print()?;
        }
        Ok(())
    }

    /// Prints content as is.
    pub fn print_historic(content: &str, mut printer: Printer<NetworkDriver>) -> Result<()> {
        printer.writeln(content)?.print_cut()?;
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
    let printer = Printer::new(
        driver,
        Protocol::default(),
        Some(PrinterOptions::new(
            Some(escpos::utils::PageCode::PC437),
            Some(DebugMode::Hex), // set to None to disable debug
            CPL,
        )),
    );

    Ok(printer)
}
