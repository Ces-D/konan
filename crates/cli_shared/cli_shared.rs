use chrono::{DateTime, Datelike, Days, Duration, Local, Months, Utc, Weekday};
use clap::{Parser, Subcommand};

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
impl From<DateBanner> for chrono::DateTime<Local> {
    fn from(val: DateBanner) -> Self {
        match val {
            DateBanner::Today => chrono::Local::now(),
            DateBanner::Tomorrow => chrono::Local::now() + chrono::Duration::days(1),
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

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default)]
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

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum RemoteFile {
    Markdown,
    Text,
}
impl RemoteFile {
    pub fn file_name(&self) -> String {
        match self {
            RemoteFile::Markdown => "konan_print.md".to_string(),
            RemoteFile::Text => "konan_print.txt".to_string(),
        }
    }
}

/// Core crates shared across cli apps
pub const CORE_MEMBERS: [&str; 4] = ["rongta", "tiptap", "blueprint", "cli_shared"];

pub fn init_logging(package: &str) {
    // Get global log level from env or use default
    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    // Build filter string applying the same level to all modules
    let mut filters = CORE_MEMBERS
        .iter()
        .map(|m| format!("{m}={level}"))
        .collect::<Vec<_>>()
        .join(",");
    filters.push_str(&format!(",{}={level}", package));

    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", &filters)
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_env(env).init();

    log::warn!("Logging initialized with level: {level}");
}
