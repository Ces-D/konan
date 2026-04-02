use anyhow::Result;
use rongta::SupportedDriver;

pub struct TextInterpreter;

impl TextInterpreter {
    pub fn print(content: &str, cut: bool, driver: SupportedDriver) -> Result<()> {
        let mut printer = rongta::build_any_printer(driver)?;
        printer.write(content)?;
        match cut {
            true => printer.print_cut()?,
            false => printer.print()?,
        }
        log::info!("Text content printed");
        Ok(())
    }
}
