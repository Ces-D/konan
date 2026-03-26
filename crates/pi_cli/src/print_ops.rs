use blueprint::interpreter::{markdown::MarkdownInterpreter, text::TextInterpreter};
use blueprint::template::{
    box_outline::BoxTemplateBuilder, get_random_box_pattern,
    habit_tracker::HabitTrackerTemplateBuilder,
};
use chrono::{DateTime, Local, Utc};
use cli_shared::RemoteFile;
use rongta::{RongtaPrinter, SupportedDriver};

const VENDOR_ID: u16 = 0x0FE6;
const PRODUCT_ID: u16 = 0x811E;

fn driver() -> SupportedDriver {
    SupportedDriver::Usb(VENDOR_ID, PRODUCT_ID)
}

pub fn print_box_template(
    cut: bool,
    rows: Option<u32>,
    lined: bool,
    banner: Option<String>,
    date: Option<DateTime<Local>>,
) -> anyhow::Result<()> {
    let pattern = get_random_box_pattern()?;
    let builder = RongtaPrinter::new(cut);
    let mut template = BoxTemplateBuilder::new(builder, pattern);
    template
        .set_rows(rows.unwrap_or(29))
        .set_lined(lined)
        .set_banner(banner);
    if let Some(d) = date {
        template.set_date_banner(d);
    }
    template.print(driver())
}

pub fn print_habit_tracker(
    cut: bool,
    habit: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> anyhow::Result<()> {
    let pattern = get_random_box_pattern()?;
    let builder = RongtaPrinter::new(cut);
    let mut template =
        HabitTrackerTemplateBuilder::new(builder, pattern, habit, start_date, end_date);
    template.print(driver())
}

pub fn print_markdown(cut: bool, content: &str, rows: Option<u32>) -> anyhow::Result<()> {
    let mut interpreter = MarkdownInterpreter::new(RongtaPrinter::new(cut));
    interpreter.print(content, rows, driver())
}

pub fn print_text(cut: bool, content: &str, rows: Option<u32>) -> anyhow::Result<()> {
    let mut interpreter = TextInterpreter::new(RongtaPrinter::new(cut));
    interpreter.print(content, rows, driver())
}

pub fn print_file(file: RemoteFile, cut: bool, rows: Option<u32>) -> anyhow::Result<()> {
    let file_content = std::fs::read_to_string(file.file_name())?;
    match file {
        RemoteFile::Markdown => print_markdown(cut, &file_content, rows),
        RemoteFile::Text => print_text(cut, &file_content, rows),
    }
}
