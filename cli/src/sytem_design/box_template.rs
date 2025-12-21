use super::BoxPattern;
use anyhow::Result;
use chrono::{DateTime, Datelike, Local, Weekday};
use rongta::{Justify, PrintBuilder, TextDecoration};

/// Calculate the next occurrence of a given weekday.
/// If today is that weekday, returns next week's occurrence.
fn next_weekday(target: Weekday) -> DateTime<Local> {
    let now = Local::now();
    let current = now.weekday().num_days_from_monday();
    let target_day = target.num_days_from_monday();
    let days_until = (target_day as i64 - current as i64 + 7) % 7;
    let days_until = if days_until == 0 { 7 } else { days_until };
    now + chrono::Duration::days(days_until)
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default)]
pub enum DateBanner {
    #[default]
    Today,
    Tomorrow,
    /// Next Monday
    Mon,
    /// Next Tuesday
    Tue,
    /// Next Wednesday
    Wed,
    /// Next Thursday
    Thu,
    /// Next Friday
    Fri,
    /// Next Saturday
    Sat,
    /// Next Sunday
    Sun,
}

pub struct BoxTemplateBuilder {
    builder: PrintBuilder,
    date: Option<DateBanner>,
    banner: Option<String>,
    rows: u32,
    lined: bool,
    pattern: BoxPattern,
}

impl BoxTemplateBuilder {
    pub fn new(builder: PrintBuilder, pattern: BoxPattern) -> Self {
        Self {
            builder,
            date: None,
            banner: None,
            rows: 30,
            lined: false,
            pattern,
        }
    }

    pub fn set_date_banner(&mut self, date: Option<DateBanner>) -> &mut Self {
        self.date = date;
        self
    }

    // Add a centered banner with the date
    fn with_date_banner(&mut self) -> Result<()> {
        match self.date {
            Some(d) => {
                let date = match d {
                    DateBanner::Today => chrono::Local::now(),
                    DateBanner::Tomorrow => chrono::Local::now() + chrono::Duration::days(1),
                    DateBanner::Mon => next_weekday(chrono::Weekday::Mon),
                    DateBanner::Tue => next_weekday(chrono::Weekday::Tue),
                    DateBanner::Wed => next_weekday(chrono::Weekday::Wed),
                    DateBanner::Thu => next_weekday(chrono::Weekday::Thu),
                    DateBanner::Fri => next_weekday(chrono::Weekday::Fri),
                    DateBanner::Sat => next_weekday(chrono::Weekday::Sat),
                    DateBanner::Sun => next_weekday(chrono::Weekday::Sun),
                };
                self.builder.set_justify_content(Justify::Center);
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    underline: true,
                    ..Default::default()
                });
                self.builder.set_text_size(rongta::TextSize::Medium);
                let str_date = date.format("%A, %B %d, %Y").to_string();
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
        match &self.banner {
            Some(b) => {
                self.builder.set_justify_content(Justify::Center);
                self.builder.set_text_decoration(TextDecoration {
                    bold: true,
                    ..Default::default()
                });
                self.builder.set_text_size(rongta::TextSize::Large);
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
        self.with_text_banner()?;
        self.with_date_banner()?;
        self.with_top()?;
        self.with_rows()?;
        self.with_bottom()?;
        self.builder.print(None)?;
        log::info!("Printed box template");
        Ok(())
    }
}
