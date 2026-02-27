use anyhow::Result;
use rongta::{RongtaPrinter, SupportedDriver};

pub struct TextInterpreter {
    builder: RongtaPrinter,
}
impl TextInterpreter {
    pub fn new(builder: RongtaPrinter) -> Self {
        Self { builder }
    }

    pub fn print(
        &mut self,
        content: &str,
        rows: Option<u32>,
        driver: SupportedDriver,
    ) -> Result<()> {
        self.builder.add_content(content)?;
        self.builder.print(rows, driver)?;
        log::info!("Text content printed");
        Ok(())
    }
}
