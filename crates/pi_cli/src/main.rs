use clap::{Parser, Subcommand};
mod connect_command;
mod file_command;
mod shared;
mod template_command;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Subscribe to IoTCore topic")]
    Connect,
    #[clap(about = "Print a file")]
    File(file_command::FileArgs),
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
    match app.command {
        Commands::Connect => connect_command::handle_connect_command().await,
        Commands::File(file_args) => {
            file_command::handle_file_command(file_args, !app.no_cut).await
        }
        Commands::Template(template_args) => {
            template_command::handle_template_command(template_args, !app.no_cut).await
        }
    }
}
