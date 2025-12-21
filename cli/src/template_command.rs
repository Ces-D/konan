use crate::sytem_design::{
    box_template::{BoxTemplateBuilder, DateBanner},
    habit_tracker_template::{HabitTrackerTemplateBuilder, TimePeriod},
};
use anyhow::Context;
use clap::{Parser, Subcommand};
use rongta::PrintBuilder;

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[clap(about = "Create a box with random borders")]
    Box {
        #[clap(
            help = "The height of the box in rows. 1in ~= 8rows.",
            default_value = "29"
        )]
        rows: Option<u32>,
        #[clap(short, long, help = "Add a date to the top of the template")]
        date: Option<DateBanner>,
        #[clap(short, long, help = "Add a message to the top of the template")]
        banner: Option<String>,
        #[clap(short, long, help = "Print a lined piece of paper")]
        lined: bool,
    },
    #[clap(about = "Create a habit tracker template")]
    HabitTracker {
        #[clap(help = "The habit to track")]
        habit: String,
        #[clap(
            short,
            long,
            help = "Start date in YYYY-MM-DD format (defaults to today)"
        )]
        start_date: Option<String>,
        #[clap(
            short,
            long,
            help = "The time period to track over",
            default_value = "month"
        )]
        time_period: Option<TimePeriod>,
    },
}

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[clap(subcommand)]
    pub command: TemplateCommand,
}

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<()> {
    match args.command {
        TemplateCommand::Box {
            rows,
            lined,
            date,
            banner,
        } => {
            let pattern = crate::sytem_design::get_random_box_pattern()?;
            let builder = PrintBuilder::new(cut);
            let mut template = BoxTemplateBuilder::new(builder, pattern);
            template
                .set_rows(rows.unwrap_or(29))
                .set_lined(lined)
                .set_banner(banner)
                .set_date_banner(date);
            template.print()?;
        }
        TemplateCommand::HabitTracker {
            habit,
            start_date,
            time_period,
        } => {
            let pattern = crate::sytem_design::get_random_box_pattern()?;
            let builder = PrintBuilder::new(cut);
            let start = if let Some(date_str) = start_date {
                chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .context("Invalid date format. Expected YYYY-MM-DD")?
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            } else {
                chrono::Utc::now()
            };
            let mut template = HabitTrackerTemplateBuilder::new(
                builder,
                pattern,
                habit,
                start,
                time_period.unwrap_or_default(),
            );
            template.print()?;
        }
    }
    Ok(())
}
