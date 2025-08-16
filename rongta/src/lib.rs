use anyhow::{Result, anyhow, ensure};
use ascii::AsciiString;
use escpos::driver::NetworkDriver;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::{DebugMode, JustifyMode, Protocol, UnderlineMode};

const CPL: u8 = 48; // characters per line
const IP: &str = "192.168.1.87";
const PORT: u16 = 9100;

#[derive(
    Debug,
    Default,
    strum::VariantNames,
    strum::EnumString,
    strum::AsRefStr,
    strum::Display,
    Clone,
    Copy,
)]
#[strum(serialize_all = "kebab-case")]
pub enum TemplateVariation {
    #[default]
    Raw,
    Heading,
}

#[derive(Debug, Default)]
pub struct Template {
    pub content: Vec<String>,
    pub min_lines: u8,
    pub variation: TemplateVariation,
}

pub fn establish_rongta_printer() -> Result<Printer<NetworkDriver>> {
    // 1) Open network driver
    let driver = match NetworkDriver::open(IP, PORT, None) {
        Ok(driver) => Ok(driver),
        Err(e) => {
            eprintln!("{}", e);
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

pub fn print(template: Template, mut printer: Printer<NetworkDriver>) -> Result<()> {
    match template.variation {
        TemplateVariation::Raw => {
            ensure!(
                template.content.len() > 0,
                "You must provide some content to print"
            );

            printer.init()?.justify(JustifyMode::LEFT)?;

            for item in template.content.iter() {
                let ascii_content = ascii_only(item)?;
                printer.writeln(ascii_content.as_str())?;
            }
            printer.print_cut()?;
            Ok(())
        }
        TemplateVariation::Heading => {
            ensure!(
                template.content.len() > 0,
                "You must provide some content to print"
            );

            printer
                .init()?
                .justify(JustifyMode::CENTER)?
                .underline(UnderlineMode::Single)?
                .bold(true)?;

            for item in template.content.iter() {
                let ascii_content = ascii_only(item)?;
                printer.writeln(ascii_content.as_str())?;
            }

            printer.print_cut()?;
            Ok(())
        }
    }
}

/// Ensure a &str is ASCII, returning an AsciiString (error if not).
fn ascii_only<S: AsRef<str>>(s: S) -> Result<AsciiString> {
    AsciiString::from_ascii(s.as_ref()).map_err(|e| anyhow!("Non-ASCII input: {}", e))
}
