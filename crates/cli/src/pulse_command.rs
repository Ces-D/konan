use crate::{command_builder::PiCommandBuilder, network::Network};
use anyhow::Result;
use clap::{Parser, Subcommand};
use cli_shared::{PulseRecipe, file_command::FileArgs, tasks, template_command::TemplateCommand};

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
    AddTemplate(cli_shared::template_command::TemplateArgs),
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

            let recipe = match template_args.command {
                TemplateCommand::Box {
                    rows,
                    date,
                    banner,
                    lined,
                } => PulseRecipe::BoxTemplate(tasks::BoxTemplatePulseRecipe {
                    cut,
                    rows,
                    lined,
                    banner,
                    date,
                }),
                TemplateCommand::HabitTracker {
                    habit, time_period, ..
                } => PulseRecipe::HabitTracker(tasks::HabitTrackerPulseRecipe {
                    cut,
                    habit,
                    time_period: time_period.unwrap_or_default(),
                }),
            };
            let command_json = recipe.to_json()?;
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

            let filename = conn.upload_file(&file_args.path, false)?;
            let recipe = PulseRecipe::File(tasks::KonanFile {
                cut,
                name: filename,
                rows: file_args.rows,
                prehook_command: file_args.prehook_command,
                prehook_command_arg: file_args.prehook_command_args,
            });

            let command_json = recipe.to_json()?;
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
