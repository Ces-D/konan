use anyhow::Context;
use serde::{Deserialize, Serialize};

pub mod clap_enum;
pub mod tasks;
pub mod template_command;
pub use template_command::TemplateArgs;

/// Direct data passed to enqueue print process
pub enum PrintTask {
    BoxTemplate(tasks::BoxTemplate),
    HabitTracker(tasks::HabitTrackerTemplate),
    Markdown(tasks::DirectPrintOut),
    Text(tasks::DirectPrintOut),
    File(tasks::KonanFile),
}

/// Tagged enum for pulse recipes that can round-trip through JSON in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PulseRecipe {
    BoxTemplate(tasks::BoxTemplatePulseRecipe),
    HabitTracker(tasks::HabitTrackerPulseRecipe),
    File(tasks::KonanFile),
}

impl PulseRecipe {
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string(self).context("Failed to serialize PulseRecipe")
    }

    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        serde_json::from_str(s).context("Failed to deserialize PulseRecipe")
    }
}

impl From<PulseRecipe> for PrintTask {
    fn from(recipe: PulseRecipe) -> Self {
        match recipe {
            PulseRecipe::BoxTemplate(r) => PrintTask::BoxTemplate(r.into()),
            PulseRecipe::HabitTracker(r) => PrintTask::HabitTracker(r.into()),
            PulseRecipe::File(r) => PrintTask::File(r),
        }
    }
}

/// Relative path (from home directory) to the konan app storage directory.
pub const APPLICATION_STORAGE_DIR: &str = ".local/share/konan";

/// Core crates shared across cli apps
pub const CORE_MEMBERS: [&str; 3] = ["rongta", "blueprint", "cli_shared"];

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
