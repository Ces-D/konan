use crate::shared::driver;
use anyhow::Context;
use blueprint::template::{
    box_outline::BoxTemplateBuilder, get_random_box_pattern,
    habit_tracker::HabitTrackerTemplateBuilder,
};
use cli_shared::{TemplateArgs, TemplateCommand};
use rongta::RongtaPrinter;

pub async fn handle_template_command(args: TemplateArgs, cut: bool) -> anyhow::Result<()> {
    match args.command {
        TemplateCommand::Box {
            rows,
            lined,
            date,
            banner,
        } => {
            let pattern = get_random_box_pattern()?;
            let builder = RongtaPrinter::new(cut);
            let mut template = BoxTemplateBuilder::new(builder, pattern);
            template
                .set_rows(rows.unwrap_or(29))
                .set_lined(lined)
                .set_banner(banner);
            if let Some(d) = date {
                template.set_date_banner(d.into());
            }
            template.print(driver())?;
        }
        TemplateCommand::HabitTracker {
            habit,
            start_date,
            time_period,
        } => {
            let pattern = get_random_box_pattern()?;
            let builder = RongtaPrinter::new(cut);
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
                time_period.unwrap_or_default().into_end_date(start),
            );
            template.print(driver())?;
        }
    }
    Ok(())
}
