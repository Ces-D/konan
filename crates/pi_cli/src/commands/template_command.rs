use crate::print_ops::enqueue_print;
use chrono::{NaiveDate, TimeZone, Utc};
use cli_shared::{
    clap_enum::TimePeriod, tasks::HabitTrackerTemplate, template_command::TemplateArgs,
};

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<String> {
    match args.command {
        cli_shared::template_command::TemplateCommand::Box {
            rows,
            date,
            banner,
            lined,
        } => {
            enqueue_print(cli_shared::PrintTask::BoxTemplate(
                cli_shared::tasks::BoxTemplate {
                    cut,
                    rows,
                    lined,
                    banner,
                    date: date.map(|v| v.into()),
                },
            ))
            .await;
            Ok("Box Template printed successfully.".to_string())
        }
        cli_shared::template_command::TemplateCommand::HabitTracker {
            habit,
            start_date,
            time_period,
        } => {
            let start_date = match start_date {
                Some(s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .map(|d| Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()))
                    .map_err(|_| anyhow::anyhow!("Invalid date format. Expected YYYY-MM-DD"))?,
                None => Utc::now(),
            };
            enqueue_print(cli_shared::PrintTask::HabitTracker(HabitTrackerTemplate {
                cut,
                habit,
                start_date,
                end_date: TimePeriod::into_datetime(time_period.unwrap_or_default(), start_date),
            }))
            .await;
            Ok("Habit Tracker printed successfully.".to_string())
        }
    }
}
