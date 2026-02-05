use super::BoxPattern;
use anyhow::Result;
use chrono::{DateTime, Datelike, Days, Duration, Utc};
use rongta::{
    RongtaPrinter, SupportedDriver,
    elements::{Justify, TextDecoration, TextSize},
};

pub struct HabitTrackerTemplateBuilder {
    builder: RongtaPrinter,
    habit: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    pattern: BoxPattern,
}

impl HabitTrackerTemplateBuilder {
    pub fn new(
        builder: RongtaPrinter,
        pattern: BoxPattern,
        habit: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Self {
        Self {
            builder,
            habit,
            start_date,
            end_date,
            pattern,
        }
    }

    fn with_time_period(&mut self) -> Result<()> {
        self.builder.new_line();
        self.builder.set_justify_content(Justify::Center);
        self.builder.set_text_decoration(TextDecoration {
            bold: true,
            underline: true,
            ..Default::default()
        });
        let start_str = self.start_date.format("%B %d, %Y").to_string();
        let end_str = self.end_date.format("%B %d, %Y").to_string();
        self.builder
            .add_content(&format!("{} - {}", start_str, end_str))?;
        self.builder.new_line();
        Ok(())
    }

    fn with_top(&mut self) -> Result<()> {
        self.builder.set_justify_content(Justify::Left);
        self.builder.set_text_decoration(TextDecoration {
            bold: true,
            ..Default::default()
        });
        self.builder.set_text_size(TextSize::Medium);
        self.builder.add_content(&self.pattern.top)?;
        self.builder.new_line();
        Ok(())
    }

    fn with_habit(&mut self) -> Result<()> {
        self.builder.set_justify_content(Justify::Center);
        self.builder.set_text_size(TextSize::Large);
        self.builder.add_content(&self.habit.to_ascii_uppercase())?;
        self.builder.new_line();
        Ok(())
    }

    fn with_checkmarks(&mut self) -> Result<()> {
        self.builder.set_justify_content(Justify::Center);
        self.builder.set_text_decoration(TextDecoration::default());
        self.builder.set_text_size(TextSize::Medium);

        const SEGMENTS_PER_LINE: usize = 4; // Max segments that fit in 48 chars with spacing

        let mut current_date = self.start_date;
        let mut day_numbers = Vec::new();

        // Collect all day numbers from start to end
        while current_date
            < self
                .end_date
                .checked_add_days(Days::new(1))
                .expect("End date overflow")
        {
            day_numbers.push(current_date.day());
            current_date = current_date
                .checked_add_days(Days::new(1))
                .unwrap_or(current_date + Duration::days(1));
        }

        // Process days in chunks and create lines
        for chunk in day_numbers.chunks(SEGMENTS_PER_LINE) {
            let line = chunk
                .iter()
                .map(|day| format!("( {:02} )", day))
                .collect::<Vec<_>>()
                .join("      ");
            self.builder.add_content(&line)?;
            self.builder.new_line();
        }

        Ok(())
    }

    fn with_bottom(&mut self) -> Result<()> {
        self.builder.set_justify_content(Justify::Left);
        self.builder.set_text_size(TextSize::Medium);
        self.builder.add_content(&self.pattern.bottom)?;
        self.builder.new_line();
        Ok(())
    }

    pub fn print(&mut self, driver: SupportedDriver) -> Result<()> {
        self.with_time_period()?;
        self.with_top()?;
        self.with_habit()?;
        self.with_top()?;
        self.with_checkmarks()?;
        self.with_bottom()?;
        self.builder.print(None, driver)?;
        log::info!("Printed habit tracker template");
        Ok(())
    }
}
