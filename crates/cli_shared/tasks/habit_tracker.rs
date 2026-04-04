use crate::clap_enum::TimePeriod;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitTrackerTemplate {
    #[serde(default = "super::default_true")]
    pub cut: bool,
    pub habit: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

impl From<HabitTrackerPulseRecipe> for HabitTrackerTemplate {
    fn from(value: HabitTrackerPulseRecipe) -> Self {
        Self {
            cut: value.cut,
            habit: value.habit,
            start_date: Utc::now(),
            end_date: value.time_period.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitTrackerPulseRecipe {
    pub cut: bool,
    pub habit: String,
    pub time_period: TimePeriod,
}
