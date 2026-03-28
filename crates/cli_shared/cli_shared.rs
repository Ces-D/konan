mod template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub use template::{DateBanner, TemplateArgs, TemplateCommand, TimePeriod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrintTask {
    BoxTemplate {
        cut: bool,
        rows: Option<u32>,
        lined: bool,
        banner: Option<String>,
        date: Option<DateTime<Utc>>,
    },
    HabitTracker {
        cut: bool,
        habit: String,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    },
    Markdown {
        cut: bool,
        content: String,
        rows: Option<u32>,
    },
    Text {
        cut: bool,
        content: String,
        rows: Option<u32>,
    },
    PulseFile {
        cut: bool,
        filename: String,
        rows: Option<u32>,
    },
    File {
        file: RemoteFile,
        cut: bool,
        rows: Option<u32>,
    },
}

impl From<PrintTask> for String {
    fn from(job: PrintTask) -> Self {
        serde_json::to_string(&job).expect("failed to serialize PrintTask")
    }
}

impl TryFrom<String> for PrintTask {
    type Error = serde_json::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&s)
    }
}

impl TryFrom<&str> for PrintTask {
    type Error = serde_json::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(s)
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

/// Relative path (from home directory) to the konan app storage directory.
pub const APPLICATION_STORAGE_DIR: &str = ".local/share/konan";
/// Relative path (from home directory) to the pulse files directory.
pub const PI_CLI_PULSE_DIR: &str = "pulse_files";

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
