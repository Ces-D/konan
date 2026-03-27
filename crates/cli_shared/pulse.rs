use crate::{DateBanner, TimePeriod};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrintJob {
    BoxTemplate {
        rows: Option<u32>,
        lined: bool,
        banner: Option<String>,
        date: Option<DateBanner>,
    },
    HabitTracker {
        habit: String,
        start_date: Option<String>, // YYYY-MM-DD; None = "today at print time"
        time_period: Option<TimePeriod>,
    },
    File {
        filename: String, // filename within ~/.config/konan/recurrence_files/
        rows: Option<u32>,
    },
}

impl From<PrintJob> for String {
    fn from(job: PrintJob) -> Self {
        serde_json::to_string(&job).expect("failed to serialize PrintJob")
    }
}

impl TryFrom<String> for PrintJob {
    type Error = serde_json::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&s)
    }
}

impl TryFrom<&str> for PrintJob {
    type Error = serde_json::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(s)
    }
}
