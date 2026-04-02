use crate::config::{printer_files_dir_path, printer_lock_path};
use anyhow::{Context, bail};
use blueprint::{
    interpreter::{markdown::MarkdownInterpreter, text::TextInterpreter},
    template::{
        box_outline::BoxTemplateBuilder, get_random_box_pattern,
        habit_tracker::HabitTrackerTemplateBuilder,
    },
};
use cli_shared::{
    PrintTask,
    tasks::{BoxTemplate, DirectPrintOut, HabitTrackerTemplate, KonanFile},
};
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
                PrintTask::BoxTemplate(template) => print_box_template(template),
                PrintTask::HabitTracker(template) => print_habit_tracker(template),
                PrintTask::Markdown(template) => print_markdown(template),
                PrintTask::Text(template) => print_text(template),
                PrintTask::File(template) => print_file(template),
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
    let lock_path = printer_lock_path()?;
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

fn print_markdown(arg: DirectPrintOut) -> anyhow::Result<()> {
    let mut interpreter = MarkdownInterpreter::new(RongtaPrinter::new(arg.cut));
    interpreter.print(&arg.content, arg.rows, driver())
}

fn print_text(arg: DirectPrintOut) -> anyhow::Result<()> {
    TextInterpreter::print(&arg.content, arg.cut, driver())
}

fn print_box_template(arg: BoxTemplate) -> anyhow::Result<()> {
    let pattern = get_random_box_pattern()?;
    let builder = RongtaPrinter::new(arg.cut);
    let mut template = BoxTemplateBuilder::new(builder, pattern);
    template
        .set_rows(arg.rows.unwrap_or(29))
        .set_lined(arg.lined)
        .set_banner(arg.banner);
    if let Some(d) = arg.date {
        template.set_date_banner(d);
    }
    template.print(driver())
}

fn print_habit_tracker(arg: HabitTrackerTemplate) -> anyhow::Result<()> {
    let pattern = get_random_box_pattern()?;
    let builder = RongtaPrinter::new(arg.cut);
    let mut template =
        HabitTrackerTemplateBuilder::new(builder, pattern, arg.habit, arg.start_date, arg.end_date);
    template.print(driver())
}

fn print_file(arg: KonanFile) -> anyhow::Result<()> {
    let file_path = printer_files_dir_path()?.join(arg.name);
    let content = std::fs::read_to_string(&file_path)
        .with_context(|| format!("Failed to read pulse file '{}'", file_path.display()))?;
    let file_extension = file_path
        .extension()
        .expect("Supported files are markdown and text");

    if file_extension == "md" {
        print_markdown(DirectPrintOut {
            cut: arg.cut,
            content,
            rows: arg.rows,
        })
    } else if file_extension == "txt" {
        print_text(DirectPrintOut {
            cut: arg.cut,
            content,
            rows: arg.rows,
        })
    } else {
        bail!("Supported extensions are markdown and text")
    }
}
