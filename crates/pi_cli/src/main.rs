use clap::{Parser, Subcommand};

use crate::config::Config;
mod commands;
mod config;
pub(crate) mod print_ops;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Subscribe to IoTCore topic")]
    Connect,
    #[clap(about = "Print a file")]
    File(commands::FileArgs),
    #[clap(about = "Print a predefined template")]
    Template(cli_shared::TemplateArgs),
}

#[derive(Debug, clap::Parser)]
#[clap(author, version, bin_name = "konan_pi", subcommand_required = true)]
pub struct App {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(
        short,
        long,
        help = "Cut or not to cut.",
        long_help = "The `rows` arg in commands ignores this flag",
        global = true
    )]
    no_cut: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli_shared::init_logging("pi_cli");
    let app = App::parse();
    let config = Config::get()?;
    match app.command {
        Commands::Connect => commands::handle_connect_command(config.connect.clone()).await,
        Commands::File(file_args) => commands::handle_file_command(file_args, !app.no_cut).await,
        Commands::Template(template_args) => {
            commands::handle_template_command(template_args, !app.no_cut).await
        }
    }
}
