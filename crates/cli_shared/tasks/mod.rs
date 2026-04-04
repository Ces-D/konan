use serde::{Deserialize, Serialize};

mod box_template;
pub use box_template::{BoxTemplate, BoxTemplatePulseRecipe};
mod file;
pub use file::KonanFile;
mod habit_tracker;
pub use habit_tracker::{HabitTrackerPulseRecipe, HabitTrackerTemplate};

pub(crate) fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPrintOut {
    #[serde(default = "default_true")]
    pub cut: bool,
    pub content: String,
    pub rows: Option<u32>,
}
