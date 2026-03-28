use crate::PrintTask;
use anyhow::Context;
use chrono::{DateTime, Datelike, Days, Duration, Months, Utc, Weekday};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, Serialize, Deserialize)]
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
impl DateBanner {
    /// Calculate the next occurrence of a given weekday.
    /// If today is that weekday, returns next week's occurrence.
    fn next_weekday(target: Weekday) -> DateTime<Utc> {
        let now = Utc::now();
        let current = now.weekday().num_days_from_monday();
        let target_day = target.num_days_from_monday();
        let days_until = (target_day as i64 - current as i64 + 7) % 7;
        let days_until = if days_until == 0 { 7 } else { days_until };
        now + chrono::Duration::days(days_until)
    }
}
impl From<DateBanner> for chrono::DateTime<Utc> {
    fn from(val: DateBanner) -> Self {
        match val {
            DateBanner::Today => chrono::Utc::now(),
            DateBanner::Tomorrow => chrono::Utc::now() + chrono::Duration::days(1),
            DateBanner::Mon => DateBanner::next_weekday(chrono::Weekday::Mon),
            DateBanner::Tue => DateBanner::next_weekday(chrono::Weekday::Tue),
            DateBanner::Wed => DateBanner::next_weekday(chrono::Weekday::Wed),
            DateBanner::Thu => DateBanner::next_weekday(chrono::Weekday::Thu),
            DateBanner::Fri => DateBanner::next_weekday(chrono::Weekday::Fri),
            DateBanner::Sat => DateBanner::next_weekday(chrono::Weekday::Sat),
            DateBanner::Sun => DateBanner::next_weekday(chrono::Weekday::Sun),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum TimePeriod {
    Week,
    #[default]
    TwoWeek,
    Month,
}
impl TimePeriod {
    pub fn into_end_date(&self, start_date: DateTime<Utc>) -> DateTime<Utc> {
        match *self {
            TimePeriod::Week => start_date
                .checked_add_days(Days::new(7))
                .unwrap_or(start_date + Duration::weeks(1)),
            TimePeriod::TwoWeek => start_date
                .checked_add_days(Days::new(14))
                .unwrap_or(start_date + Duration::weeks(2)),
            TimePeriod::Month => start_date
                .checked_add_months(Months::new(1))
                .unwrap_or(start_date + Duration::days(30)),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[clap(about = "Create a box with random borders")]
    Box {
        #[clap(
            long,
            help = "The height of the box in rows. 1in ~= 8rows.",
            default_value = "29"
        )]
        rows: Option<u32>,
        #[clap(short, long, help = "Add a date to the top of the template")]
        date: Option<DateBanner>,
        #[clap(short, long, help = "Add a message to the top of the template")]
        banner: Option<String>,
        #[clap(short, long, help = "Print a lined piece of paper")]
        lined: bool,
    },
    #[clap(about = "Create a habit tracker template")]
    HabitTracker {
        #[clap(help = "The habit to track")]
        habit: String,
        #[clap(
            short,
            long,
            help = "Start date in YYYY-MM-DD format (defaults to today)"
        )]
        start_date: Option<String>,
        #[clap(
            short,
            long,
            help = "The time period to track over",
            default_value = "two-week"
        )]
        time_period: Option<TimePeriod>,
    },
}

impl TemplateCommand {
    pub fn into_print_task(self, cut: bool) -> anyhow::Result<PrintTask> {
        match self {
            TemplateCommand::Box {
                rows,
                lined,
                date,
                banner,
            } => Ok(PrintTask::BoxTemplate {
                cut,
                rows,
                lined,
                banner,
                date: date.map(|d| d.into()),
            }),
            TemplateCommand::HabitTracker {
                habit,
                start_date,
                time_period,
            } => {
                let start = if let Some(date_str) = start_date {
                    chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .context("Invalid date format. Expected YYYY-MM-DD")?
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_utc()
                } else {
                    chrono::Utc::now()
                };
                let end = time_period.unwrap_or_default().into_end_date(start);
                Ok(PrintTask::HabitTracker {
                    cut,
                    habit,
                    start_date: start,
                    end_date: end,
                })
            }
        }
    }
}

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[clap(subcommand)]
    pub command: TemplateCommand,
}
