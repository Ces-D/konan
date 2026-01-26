use anyhow::Context;
use chrono::{DateTime, Datelike, Days, Duration, Local, Months, Utc, Weekday};
use clap::{Parser, Subcommand};
use designs::{
    box_template::BoxTemplateBuilder, habit_tracker_template::HabitTrackerTemplateBuilder,
};
use rongta::PrintBuilder;

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
impl DateBanner {
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
}
impl Into<chrono::DateTime<Local>> for DateBanner {
    fn into(self) -> DateTime<Local> {
        match self {
            DateBanner::Today => chrono::Local::now(),
            DateBanner::Tomorrow => chrono::Local::now() + chrono::Duration::days(1),
            DateBanner::Mon => Self::next_weekday(chrono::Weekday::Mon),
            DateBanner::Tue => Self::next_weekday(chrono::Weekday::Tue),
            DateBanner::Wed => Self::next_weekday(chrono::Weekday::Wed),
            DateBanner::Thu => Self::next_weekday(chrono::Weekday::Thu),
            DateBanner::Fri => Self::next_weekday(chrono::Weekday::Fri),
            DateBanner::Sat => Self::next_weekday(chrono::Weekday::Sat),
            DateBanner::Sun => Self::next_weekday(chrono::Weekday::Sun),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default)]
pub enum TimePeriod {
    Week,
    #[default]
    TwoWeek,
    Month,
}
impl TimePeriod {
    pub fn into_end_date(&self, start_date: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            &TimePeriod::Week => start_date
                .checked_add_days(Days::new(7))
                .unwrap_or(start_date + Duration::weeks(1)),
            &TimePeriod::TwoWeek => start_date
                .checked_add_days(Days::new(14))
                .unwrap_or(start_date + Duration::weeks(2)),
            &TimePeriod::Month => start_date
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

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[clap(subcommand)]
    pub command: TemplateCommand,
}

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<()> {
    match args.command {
        TemplateCommand::Box {
            rows,
            lined,
            date,
            banner,
        } => {
            let pattern = designs::get_random_box_pattern()?;
            let builder = PrintBuilder::new(cut);
            let mut template = BoxTemplateBuilder::new(builder, pattern);
            template
                .set_rows(rows.unwrap_or(29))
                .set_lined(lined)
                .set_banner(banner);
            if let Some(d) = date {
                template.set_date_banner(d.into());
            }

            template.print()?;
        }
        TemplateCommand::HabitTracker {
            habit,
            start_date,
            time_period,
        } => {
            let pattern = designs::get_random_box_pattern()?;
            let builder = PrintBuilder::new(cut);
            let start = if let Some(date_str) = start_date {
                chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .context("Invalid date format. Expected YYYY-MM-DD")?
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            } else {
                chrono::Utc::now()
            };
            let mut template = HabitTrackerTemplateBuilder::new(
                builder,
                pattern,
                habit,
                start,
                time_period.unwrap_or_default().into_end_date(start),
            );
            template.print()?;
        }
    }
    Ok(())
}
