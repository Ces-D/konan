mod art_command;
mod file_command;
mod sytem_design;
mod template_command;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Print a file")]
    File(file_command::FileArgs),
    #[clap(about = "Print ai art")]
    Art(art_command::ArtArgs),
    #[clap(about = "Print text in large format")]
    BigText(art_command::BigTextArgs),
    #[clap(about = "Print a predefined template")]
    Template(template_command::TemplateArgs),
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "konan", subcommand_required = true)]
pub struct App {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(
        short = 'n',
        long = "no-cut",
        help = "A flag identifying that the printer should not cut after printing",
        global = true
    )]
    no_cut: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();
    let app = App::parse();
    match app.command {
        Commands::File(file_args) => file_command::handle_file_command(file_args, app.no_cut).await,
        Commands::Art(art_args) => art_command::handle_art_command(art_args, app.no_cut).await,
        Commands::BigText(big_text_args) => {
            art_command::handle_big_text_command(big_text_args, app.no_cut).await
        }
        Commands::Template(template_args) => {
            template_command::handle_template_command(template_args, app.no_cut).await
        }
    }
}

fn init_logging() {
    const MEMBERS: [&str; 3] = ["cli", "ai", "rongta"];

    // Get global log level from env or use default
    let level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    // Build filter string applying the same level to all modules
    let filters = MEMBERS
        .iter()
        .map(|m| format!("{m}={level}"))
        .collect::<Vec<_>>()
        .join(",");

    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", &filters)
        .write_style_or("RUST_LOG_STYLE", "always");

    env_logger::Builder::from_env(env).init();

    log::warn!("Logging initialized with level: {level}");
}
