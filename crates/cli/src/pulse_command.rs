use crate::{command_builder::PiCommandBuilder, file_command::FileArgs, network::Network};
use anyhow::Result;
use clap::{Parser, Subcommand};
use cli_shared::{PrintTask, TemplateArgs};

#[derive(Debug, Parser)]
pub struct PulseArgs {
    #[clap(subcommand)]
    pub command: PulseDirectCommand,
    #[clap(
        short,
        long,
        help = "Name of this job. REQUIRED when adding",
        global = true
    )]
    pub name: Option<String>,
    #[clap(
        short,
        long,
        help = "Rrule of this job. REQUIRED when adding",
        global = true
    )]
    pub rrule: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum PulseDirectCommand {
    #[clap(about = "Command for scheduling template print outs")]
    AddTemplate(TemplateArgs),
    #[clap(about = "Command for scheduling file print outs")]
    AddFile(FileArgs),
    #[clap(about = "Command for deleting scheduled items")]
    Delete { id: i64 },
    #[clap(about = "Command for listing scheduled items")]
    List,
}

pub async fn handle_pulse_command(args: PulseArgs, cut: bool) -> Result<()> {
    let mut conn = Network::new()?;
    match args.command {
        PulseDirectCommand::AddTemplate(template_args) => {
            let name = args
                .name
                .ok_or_else(|| anyhow::anyhow!("--name is required when adding a pulse"))?;
            let rrule = args
                .rrule
                .ok_or_else(|| anyhow::anyhow!("--rrule is required when adding a pulse"))?;

            let print_job = template_args.command.into_print_task(cut)?;
            let command_json: String = print_job.into();
            let cmd = PiCommandBuilder::new("pulse add")
                .positional(&name)
                .positional(&rrule)
                .positional(&command_json);
            conn.execute_command(cmd)
        }
        PulseDirectCommand::AddFile(file_args) => {
            let name = args
                .name
                .ok_or_else(|| anyhow::anyhow!("--name is required when adding a pulse"))?;
            let rrule = args
                .rrule
                .ok_or_else(|| anyhow::anyhow!("--rrule is required when adding a pulse"))?;

            let filename = conn.upload_pulse_file(&file_args.path)?;
            let print_job = PrintTask::PulseFile {
                cut,
                filename,
                rows: file_args.rows,
            };

            let command_json: String = print_job.into();
            let cmd = PiCommandBuilder::new("pulse add")
                .positional(&name)
                .positional(&rrule)
                .positional(&command_json);
            conn.execute_command(cmd)
        }
        PulseDirectCommand::Delete { id } => {
            let cmd = PiCommandBuilder::new("pulse delete").positional(&id.to_string());
            conn.execute_command(cmd)
        }
        PulseDirectCommand::List => {
            let cmd = PiCommandBuilder::new("pulse list");
            conn.execute_command(cmd)
        }
    }
}
