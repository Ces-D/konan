use actix_web::{
    Result, get,
    web::{self, Json},
};
use chrono::{DateTime, Utc};
use designs::{
    box_template::BoxTemplateBuilder, habit_tracker_template::HabitTrackerTemplateBuilder,
};
use rongta::PrintBuilder;
use serde::Deserialize;

use crate::routes::Message;

#[derive(Debug, Deserialize, Default)]
struct OutlineParams {
    rows: Option<u32>,
    date: Option<DateTime<Utc>>,
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

#[get("/habit-tracker")]
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
// TODO: add endpoint that accepts document formatting and converts it into StyledChars
