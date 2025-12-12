use super::BoxPattern;
use anyhow::Result;
use rongta::{Justify, PrintBuilder, TextDecoration};

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum DateBanner {
    Today,
    Tomorrow,
}

pub struct BoxTemplateBuilder {
    builder: PrintBuilder,
    date: Option<DateBanner>,
    rows: u32,
    lined: bool,
    pattern: BoxPattern,
}

impl BoxTemplateBuilder {
    pub fn new(builder: PrintBuilder, pattern: BoxPattern) -> Self {
        Self {
            builder,
            date: None,
            rows: 28,
            lined: false,
            pattern,
        }
    }

    pub fn set_date_banner(&mut self, banner: Option<DateBanner>) -> &mut Self {
        self.date = banner;
        self
    }

    // Add a centered banner with the date
    fn with_date_banner(&mut self) -> Result<()> {
        if self.date.is_none() {
            return Ok(());
        }
        let date = match self.date.unwrap() {
            DateBanner::Today => chrono::Local::now(),
            DateBanner::Tomorrow => chrono::Local::now() + chrono::Duration::days(1),
        };
        self.builder.set_justify_content(Justify::Center);
        self.builder.set_text_decoration(TextDecoration {
            bold: true,
            underline: true,
            ..Default::default()
        });
        let str_date = date.format("%A, %B %d, %Y").to_string();
        self.builder.add_content(&str_date)?;
        self.builder.new_line();
        Ok(())
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
        self.builder.set_justify_content(Justify::Left);
        self.builder.set_text_decoration(TextDecoration {
            bold: true,
            ..Default::default()
        });
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
        self.builder.add_content(&self.pattern.top)?;
        self.builder.new_line();
        Ok(())
    }

    fn with_bottom(&mut self) -> Result<()> {
        self.builder.add_content(&self.pattern.bottom)?;
        self.builder.new_line();
        Ok(())
    }

    /// AKA build
    pub fn print(&mut self) -> Result<()> {
        self.with_date_banner()?;
        self.with_top()?;
        self.with_rows()?;
        self.with_bottom()?;
        self.builder.print()?;
        log::info!("Printed box template");
        Ok(())
    }
}
