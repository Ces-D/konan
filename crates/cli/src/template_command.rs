use crate::{command_builder::PiCommandBuilder, network::Network};
use cli_shared::{TemplateArgs, template_command::TemplateCommand};

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<()> {
    let mut conn = Network::new()?;
    match args.command {
        TemplateCommand::Box {
            rows,
            lined,
            date,
            banner,
        } => {
            let cmd = PiCommandBuilder::new("template box")
                .named("rows", rows)
                .flag("lined", lined)
                .named_enum("date", date)
                .named("banner", banner)
                .flag("no-cut", !cut);
            conn.execute_command(cmd)
        }
        TemplateCommand::HabitTracker {
            habit,
            start_date,
            time_period,
        } => {
            let cmd = PiCommandBuilder::new("template habit-tracker")
                .positional(&habit)
                .named("start-date", start_date)
                .named_enum("time-period", time_period)
                .flag("no-cut", !cut);
            conn.execute_command(cmd)
        }
    }
}
