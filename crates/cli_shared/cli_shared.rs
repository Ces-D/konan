mod template;
pub use template::{DateBanner, TemplateArgs, TemplateCommand, TimePeriod};
mod pulse;
pub use pulse::PrintJob;

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
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
pub const CORE_MEMBERS: [&str; 4] = ["rongta", "tiptap", "blueprint", "cli_shared"];

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
