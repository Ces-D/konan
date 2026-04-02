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

#[derive(clap::ValueEnum, Clone, Copy, Debug, Serialize, Deserialize)]
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
