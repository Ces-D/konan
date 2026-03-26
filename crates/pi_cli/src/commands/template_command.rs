use anyhow::Context;
use cli_shared::{TemplateArgs, TemplateCommand};

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<()> {
    match args.command {
        TemplateCommand::Box {
            rows,
            lined,
            date,
            banner,
        } => {
            let date_local = date.map(|d| d.into());
            crate::print_ops::print_box_template(cut, rows, lined, banner, date_local)
        }
        TemplateCommand::HabitTracker {
            habit,
            start_date,
            time_period,
        } => {
            let start = if let Some(date_str) = start_date {
                chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .context("Invalid date format. Expected YYYY-MM-DD")?
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
            } else {
                chrono::Utc::now()
            };
            let end = time_period.unwrap_or_default().into_end_date(start);
            crate::print_ops::print_habit_tracker(cut, habit, start, end)
        }
    }
}
