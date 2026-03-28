use crate::config::{application_storage_path, pulse_files_dir};
use anyhow::Context;
use blueprint::{
    interpreter::{markdown::MarkdownInterpreter, text::TextInterpreter},
    template::{
        box_outline::BoxTemplateBuilder, get_random_box_pattern,
        habit_tracker::HabitTrackerTemplateBuilder,
    },
};
use chrono::{DateTime, Utc};
use cli_shared::{PrintTask, RemoteFile};
use fs4::fs_std::FileExt;
use rongta::{RongtaPrinter, SupportedDriver};
use std::{fs::OpenOptions, sync::OnceLock};
use tokio::sync::mpsc;

const VENDOR_ID: u16 = 0x0FE6;
const PRODUCT_ID: u16 = 0x811E;

type PrintQueue = mpsc::Sender<PrintTask>;

static PRINT_QUEUE: OnceLock<PrintQueue> = OnceLock::new();

pub fn init_queue() {
    let (tx, mut rx) = mpsc::channel::<PrintTask>(32);
    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            let lock_file = match acquire_printer_lock() {
                Ok(f) => f,
                Err(e) => {
                    log::error!("Could not acquire printer lock, skipping job: {e:#}");
                    continue;
                }
            };

            let result = match task {
                PrintTask::BoxTemplate {
                    cut,
                    rows,
                    lined,
                    banner,
                    date,
                } => print_box_template(cut, rows, lined, banner, date),
                PrintTask::HabitTracker {
                    cut,
                    habit,
                    start_date,
                    end_date,
                } => print_habit_tracker(cut, habit, start_date, end_date),
                PrintTask::Markdown { cut, content, rows } => print_markdown(cut, &content, rows),
                PrintTask::Text { cut, content, rows } => print_text(cut, &content, rows),
                PrintTask::PulseFile {
                    cut,
                    filename,
                    rows,
                } => print_pulse_file(cut, &filename, rows),
                PrintTask::File { file, cut, rows } => print_remote_file(file, cut, rows),
            };

            if let Err(e) = lock_file.unlock() {
                log::error!("Failed to release printer lock: {e:#}");
                break;
            }

            if let Err(e) = result {
                log::error!("Print task failed: {e:#}");
            }
        }
    });
    PRINT_QUEUE
        .set(tx)
        .expect("Unable to initialize the PRINT_QUEUE")
}

pub async fn enqueue_print(task: PrintTask) {
    PRINT_QUEUE
        .get()
        .expect("PRINT_QUEUE not initialized")
        .send(task)
        .await
        .expect("PRINT_QUEUE receiver dropped");
}

fn driver() -> SupportedDriver {
    SupportedDriver::Usb(VENDOR_ID, PRODUCT_ID)
}

fn acquire_printer_lock() -> anyhow::Result<std::fs::File> {
    let lock_path = application_storage_path()?.join("printer.lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("Failed to open lock file '{}'", lock_path.display()))?;
    file.lock_exclusive()
        .context("Failed to acquire exclusive printer lock")?;
    Ok(file)
}

fn print_markdown(cut: bool, content: &str, rows: Option<u32>) -> anyhow::Result<()> {
    let mut interpreter = MarkdownInterpreter::new(RongtaPrinter::new(cut));
    interpreter.print(content, rows, driver())
}

fn print_text(cut: bool, content: &str, rows: Option<u32>) -> anyhow::Result<()> {
    let mut interpreter = TextInterpreter::new(RongtaPrinter::new(cut));
    interpreter.print(content, rows, driver())
}

fn print_box_template(
    cut: bool,
    rows: Option<u32>,
    lined: bool,
    banner: Option<String>,
    date: Option<DateTime<Utc>>,
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

fn print_habit_tracker(
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

fn print_pulse_file(cut: bool, filename: &str, rows: Option<u32>) -> anyhow::Result<()> {
    let file_path = pulse_files_dir()?.join(filename);
    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read pulse file '{}'", file_path.display()))?;
    if filename.ends_with(".md") {
        print_markdown(cut, &content, rows)
    } else {
        print_text(cut, &content, rows)
    }
}

fn print_remote_file(file: RemoteFile, cut: bool, rows: Option<u32>) -> anyhow::Result<()> {
    let remote_path = application_storage_path()?.join(file.file_name());
    let file_content = std::fs::read_to_string(remote_path)?;
    match file {
        RemoteFile::Markdown => print_markdown(cut, &file_content, rows),
        RemoteFile::Text => print_text(cut, &file_content, rows),
    }
}
