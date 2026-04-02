use crate::clap_enum::{DateBanner, TimePeriod};
use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[clap(about = "Create a box with random borders")]
    Box {
        #[clap(
            long,
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
            default_value = "two-week"
        )]
        time_period: Option<TimePeriod>,
    },
}

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[clap(subcommand)]
    pub command: TemplateCommand,
}
