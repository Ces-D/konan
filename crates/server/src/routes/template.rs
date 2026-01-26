use actix_web::{
    Result, get,
    web::{self, Json},
};
use chrono::{DateTime, Datelike, Local, Utc, Weekday};
use designs::{
    box_template::BoxTemplateBuilder, habit_tracker_template::HabitTrackerTemplateBuilder,
};
use rongta::PrintBuilder;
use serde::{Deserialize, Serialize};

use crate::routes::Message;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub enum DateBanner {
    #[default]
    #[serde()]
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

#[derive(Debug, Deserialize, Default)]
struct OutlineParams {
    rows: Option<u32>,
    date: Option<DateBanner>,
    banner: Option<String>,
    lined: Option<bool>,
}

/// Create a box with random borders
#[get("/outline")]
async fn outline(params: web::Query<OutlineParams>) -> Result<Json<Message>> {
    let pattern = designs::get_random_box_pattern()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Pattern resource failure"))?;
    let builder = PrintBuilder::new(true);
    let mut template = BoxTemplateBuilder::new(builder, pattern);
    template
        .set_rows(params.rows.unwrap_or(29))
        .set_lined(params.lined.unwrap_or_default())
        .set_banner(params.banner.clone());
    if let Some(d) = params.date {
        template.set_date_banner(d.into());
    }
    template
        .print()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to print"))?;
    Ok(Json(Message::default()))
}

#[derive(Debug, Deserialize)]
struct HabitTrackerParams {
    habit: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

#[get("/habit_tracker")]
async fn habit_tracker(params: web::Query<HabitTrackerParams>) -> Result<Json<Message>> {
    let pattern = designs::get_random_box_pattern()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Pattern resource failure"))?;
    let builder = PrintBuilder::new(true);
    let mut template = HabitTrackerTemplateBuilder::new(
        builder,
        pattern,
        params.habit.clone(),
        params.start_date,
        params.end_date,
    );
    template
        .print()
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to print"))?;
    Ok(Json(Message::default()))
}
// TODO: add these buttons to frontend
// TODO: add endpoint that accepts document formatting and converts it into StyledChars
