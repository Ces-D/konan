use super::BoxPattern;
use anyhow::Result;
use chrono::{DateTime, Local};
use rongta::{
    RongtaPrinter, SupportedDriver,
    elements::{Justify, TextSize},
};

pub struct BoxTemplateBuilder {
    builder: RongtaPrinter,
    date: Option<DateTime<Local>>,
    banner: Option<String>,
    rows: u32,
    lined: bool,
    pattern: BoxPattern,
}

impl BoxTemplateBuilder {
    pub fn new(builder: RongtaPrinter, pattern: BoxPattern) -> Self {
        Self {
            builder,
            date: None,
            banner: None,
            rows: 30,
            lined: false,
            pattern,
        }
    }

    pub fn set_date_banner(&mut self, date: DateTime<Local>) -> &mut Self {
        self.date = Some(date);
        self
    }

    // Add a centered banner with the date
    fn with_date_banner(&mut self) -> Result<()> {
        self.builder.reset_styles();
        self.builder.set_justify_content(Justify::Center);
        self.builder.set_is_bold(true);

        match self.date {
            Some(d) => {
                let str_date = d.format("%A, %B %d, %Y").to_string();
                self.builder.add_content(&str_date)?;
                self.builder.new_line();
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn set_banner(&mut self, message: Option<String>) -> &mut Self {
        self.banner = message;
        self
    }
    fn with_text_banner(&mut self) -> Result<()> {
        self.builder.reset_styles();
        match &self.banner {
            Some(b) => {
                self.builder.set_justify_content(Justify::Center);
                self.builder.set_is_bold(true);
                self.builder.set_text_size(TextSize::Large);
                self.builder.add_content(b)?;
                self.builder.new_line();
                self.builder.new_line();
                Ok(())
            }
            None => Ok(()),
        }
    }

    pub fn set_lined(&mut self, lined: bool) -> &mut Self {
        self.lined = lined;
        self
    }

    pub fn set_rows(&mut self, rows: u32) -> &mut Self {
        self.rows = rows;
        self
    }

    fn with_rows(&mut self) -> Result<()> {
        self.builder.reset_styles();
        self.builder.set_is_bold(true);
        for i in 0..self.rows {
            if self.lined {
                if i % 2 == 0 {
                    self.builder
                        .add_content(&self.pattern.row.clone().replace(" ", "."))?;
                    self.builder.new_line();
                } else {
                    self.builder.add_content(&self.pattern.row.clone())?;
                    self.builder.new_line();
                }
            } else {
                self.builder.add_content(&self.pattern.row.clone())?;
                self.builder.new_line();
            }
        }
        Ok(())
    }

    fn with_top(&mut self) -> Result<()> {
        self.builder.reset_styles();
        self.builder.set_is_bold(true);
        self.builder.add_content(&self.pattern.top)?;
        self.builder.new_line();
        Ok(())
    }

    fn with_bottom(&mut self) -> Result<()> {
        self.builder.reset_styles();
        self.builder.set_is_bold(true);
        self.builder.add_content(&self.pattern.bottom)?;
        self.builder.new_line();
        Ok(())
    }

    /// AKA build
    pub fn print(&mut self, driver: SupportedDriver) -> Result<()> {
        self.with_text_banner()?;
        self.with_date_banner()?;
        self.with_top()?;
        self.with_rows()?;
        self.with_bottom()?;
        self.builder.print(None, driver)?;
        log::info!("Printed box template");
        Ok(())
    }
}
