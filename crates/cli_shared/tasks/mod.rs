use serde::{Deserialize, Serialize};

mod box_template;
pub use box_template::{BoxTemplate, BoxTemplatePulseRecipe};
mod file;
pub use file::KonanFile;
mod habit_tracker;
pub use habit_tracker::{HabitTrackerPulseRecipe, HabitTrackerTemplate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectPrintOut {
    pub cut: bool,
    pub content: String,
    pub rows: Option<u32>,
}
