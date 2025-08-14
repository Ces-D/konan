use anyhow::{Result, anyhow};
use ascii::AsciiString;
use escpos::driver::NetworkDriver;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::{DebugMode, JustifyMode, Protocol};

const CPL: u8 = 48; // characters per line
const IP: &str = "192.168.1.87";
const PORT: u16 = 9100;

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

pub fn print(content: String, mut printer: Printer<NetworkDriver>) -> Result<()> {
    let ascii_content = ascii_only(content)?;

    // 4) Print
    printer
        .init()?
        .justify(JustifyMode::LEFT)?
        .writeln(ascii_content.as_str())?;

    printer.print_cut()?;

    Ok(())
}

/// Ensure a &str is ASCII, returning an AsciiString (error if not).
fn ascii_only<S: AsRef<str>>(s: S) -> Result<AsciiString> {
    AsciiString::from_ascii(s.as_ref()).map_err(|e| anyhow!("Non-ASCII input: {}", e))
}
