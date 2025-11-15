mod art_command;
mod file_command;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Print a file")]
    File(file_command::FileArgs),
    #[clap(about = "Print ai art")]
    Art(art_command::ArtArgs),
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
    env_logger::builder().init();
    let app = App::parse();
    match app.command {
        Commands::File(file_args) => file_command::handle_file_command(file_args, app.no_cut).await,
        Commands::Art(art_args) => art_command::handle_art_command(art_args, app.no_cut).await,
    }
}
