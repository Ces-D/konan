use crate::{
    database::{
        delete_pulse, get_all_pulses, insert_pulse,
        schema::{CompactPulse, NewPulse, Pulse, nyc_tz},
        update_last_run,
    },
    print_ops::enqueue_print,
};
use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use cli_shared::{PrintTask, PulseRecipe, tasks::DirectPrintOut};
use std::str::FromStr;

#[derive(Debug, Parser)]
pub struct PulseArgs {
    #[clap(subcommand)]
    pub command: PulseCommand,
}

#[derive(Debug, Subcommand)]
pub enum PulseCommand {
    Add {
        name: String,
        rrule: String,
        command: String,
    },
    List,
    Delete {
        id: i64,
    },
    Print,
}

pub async fn handle_pulse_command(args: PulseArgs) -> Result<String> {
    match args.command {
        PulseCommand::Print => {
            let pulses = get_all_pulses()?;
            if pulses.is_empty() {
                return Ok("No pulses configured.".to_string());
            }

            let now = Utc::now();
            let mut results: Vec<String> = Vec::new();
            for pulse in pulses {
                if !should_run(&pulse, &now) {
                    continue;
                }
                let task: PrintTask = match pulse.parsed_command() {
                    Ok(recipe) => recipe.into(),
                    Err(e) => {
                        let msg = format!("Error parsing command for pulse '{}': {e}", pulse.name);
                        print_error(&msg).await;
                        results.push(msg);
                        continue;
                    }
                };
                enqueue_print(task).await;
                if let Err(e) = update_last_run(pulse.id) {
                    let msg = format!("Error updating last_run for pulse '{}': {e}", pulse.name);
                    print_error(&msg).await;
                    results.push(msg);
                    continue;
                }
                results.push(format!("Pulse '{}' ran successfully.", pulse.name));
            }

            if results.is_empty() {
                return Ok("No pulses were due to run.".to_string());
            }
            Ok(results.join("\n"))
        }
        PulseCommand::List => {
            let pulses = get_all_pulses()?;
            if pulses.is_empty() {
                return Ok("No pulses configured.".to_string());
            }
            let mut compact: Vec<CompactPulse> = vec![];
            for pulse in pulses {
                match CompactPulse::try_from(pulse) {
                    Ok(p) => compact.push(p),
                    Err(e) => eprintln!("{}", e),
                }
            }
            serde_json::to_string_pretty(&compact)
                .with_context(|| "Unable to summarize list of pulses")
        }
        PulseCommand::Add {
            name,
            rrule,
            command,
        } => {
            let recipe = PulseRecipe::from_json(&command)
                .with_context(|| format!("Invalid pulse command JSON: {command}"))?;
            let unvalidated_rrule = rrule::RRule::from_str(&rrule)?;
            let pulse = NewPulse::new(name.clone(), recipe, unvalidated_rrule)?;
            insert_pulse(&pulse)?;
            Ok(format!("Pulse '{name}' added successfully."))
        }
        PulseCommand::Delete { id } => {
            delete_pulse(id)?;
            Ok(format!("Pulse '{id}' deleted successfully."))
        }
    }
}

async fn print_error(message: &str) {
    eprintln!("{message}");
    enqueue_print(PrintTask::Text(DirectPrintOut {
        cut: true,
        content: message.to_string(),
        rows: None,
    }))
    .await;
}

fn should_run(pulse: &Pulse, now: &chrono::DateTime<Utc>) -> bool {
    let rr = match pulse.validated_rrule() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Invalid rrule for pulse '{}': {e}", pulse.name);
            return false;
        }
    };

    let after = pulse.last_run.with_timezone(&nyc_tz());
    let before = now.with_timezone(&nyc_tz());

    let rrule_set = rrule::RRuleSet::new(pulse.start_date.with_timezone(&nyc_tz())).rrule(rr);

    !rrule_set
        .after(after)
        .before(before)
        .all(1)
        .dates
        .is_empty()
}
