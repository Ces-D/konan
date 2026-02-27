use crate::network::Network;
use anyhow::bail;
use clap::ValueEnum;
use cli_shared::{TemplateArgs, TemplateCommand};

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<()> {
    let mut conn = Network::new()?;
    match args.command {
        TemplateCommand::Box {
            rows,
            lined,
            date,
            banner,
        } => {
            let mut cmd = "konan template box".to_string();
            if let Some(rows) = rows {
                cmd.push_str(&format!(" --rows {}", rows));
            }
            if lined {
                cmd.push_str(" --lined");
            }
            if let Some(date) = date {
                cmd.push_str(&format!(
                    " --date {:?}",
                    date.to_possible_value().unwrap_or_default()
                ));
            }
            if let Some(banner) = banner {
                cmd.push_str(&format!(" --banner {}", banner));
            }
            if !cut {
                cmd.push_str(" --no-cut");
            }
            match conn.execute_command(cmd) {
                Ok(output) => {
                    println!("{}", output);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to print box template: {:?}", e);
                    bail!("Failed to print box template")
                }
            }
        }
        TemplateCommand::HabitTracker {
            habit,
            start_date,
            time_period,
        } => {
            let mut cmd = format!("konan template habit-tracker '{}'", habit);
            if let Some(start_date) = start_date {
                cmd.push_str(&format!(" --start_date {}", start_date));
            }
            if let Some(time_period) = time_period {
                cmd.push_str(&format!(
                    " --time_period {:?}",
                    time_period.to_possible_value().unwrap_or_default()
                ));
            }
            if !cut {
                cmd.push_str(" --no-cut");
            }
            match conn.execute_command(cmd) {
                Ok(output) => {
                    println!("{}", output);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to print habit tracker: {:?}", e);
                    bail!("Failed to print habit tracker")
                }
            }
        }
    }
}
