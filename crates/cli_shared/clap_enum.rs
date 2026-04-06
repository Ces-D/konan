use std::{fmt::Display, path::PathBuf};

use anyhow::{Context, bail};
use chrono::{DateTime, Datelike, Duration, Months, Utc, Weekday};
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
        now + Duration::days(days_until)
    }
}
impl From<DateBanner> for chrono::DateTime<Utc> {
    fn from(val: DateBanner) -> Self {
        match val {
            DateBanner::Today => Utc::now(),
            DateBanner::Tomorrow => Utc::now() + Duration::days(1),
            DateBanner::Mon => DateBanner::next_weekday(Weekday::Mon),
            DateBanner::Tue => DateBanner::next_weekday(Weekday::Tue),
            DateBanner::Wed => DateBanner::next_weekday(Weekday::Wed),
            DateBanner::Thu => DateBanner::next_weekday(Weekday::Thu),
            DateBanner::Fri => DateBanner::next_weekday(Weekday::Fri),
            DateBanner::Sat => DateBanner::next_weekday(Weekday::Sat),
            DateBanner::Sun => DateBanner::next_weekday(Weekday::Sun),
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
    pub fn into_datetime(
        value: TimePeriod,
        start_date: chrono::DateTime<Utc>,
    ) -> chrono::DateTime<Utc> {
        match value {
            TimePeriod::Week => start_date + chrono::Duration::weeks(1),
            TimePeriod::TwoWeek => start_date + chrono::Duration::weeks(2),
            TimePeriod::Month => start_date
                .checked_add_months(Months::new(1))
                .unwrap_or(start_date + Duration::weeks(4)),
        }
    }
}
impl From<TimePeriod> for chrono::DateTime<Utc> {
    fn from(value: TimePeriod) -> Self {
        TimePeriod::into_datetime(value, Utc::now())
    }
}

#[derive(clap::ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum AllowedCommand {
    DailyBugleNow,
    DailyBugleToday,
    DailyBugleThisWeek,
}
impl Display for AllowedCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            AllowedCommand::DailyBugleNow => "daily-bugle-now",
            AllowedCommand::DailyBugleToday => "daily-bugle-today",
            AllowedCommand::DailyBugleThisWeek => "daily-bugle-this-week",
        };
        write!(f, "{}", name)
    }
}
impl AllowedCommand {
    pub fn run_command(&self, store_loc: PathBuf, profile: &str) -> anyhow::Result<()> {
        let command = match self {
            AllowedCommand::DailyBugleNow => std::process::Command::new("daily-bugle")
                .args(["almanac", "now", "--profile", profile])
                .output(),
            AllowedCommand::DailyBugleToday => std::process::Command::new("daily-bugle")
                .args(["almanac", "today", "--profile", profile])
                .output(),
            AllowedCommand::DailyBugleThisWeek => std::process::Command::new("daily-bugle")
                .args(["almanac", "this-week", "--profile", profile])
                .output(),
        };
        let command = command
            .with_context(|| format!("Failed to execute '{self}' with profile '{profile}'"))?;
        if command.status.success() {
            std::fs::write(&store_loc, &command.stdout).with_context(|| {
                format!("Failed to write '{self}' output to {}", store_loc.display())
            })
        } else {
            let stderr = String::from_utf8_lossy(&command.stderr);
            bail!("'{self}' exited with {}\nstderr: {stderr}", command.status);
        }
    }
}
