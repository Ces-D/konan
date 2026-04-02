use crate::print_ops::enqueue_print;
use clap::Parser;
use cli_shared::{PrintTask, clap_enum::RemoteFile, tasks::KonanFile};

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(short, long, help = "Remote file to print")]
    file: RemoteFile,
    #[clap(short, long, help = "Number of rows per page (cuts after each page)")]
    rows: Option<u32>,
}

pub async fn handle_file_command(args: FileArgs, cut: bool) -> anyhow::Result<String> {
    enqueue_print(PrintTask::File(KonanFile {
        name: args.file.file_name(),
        cut,
        rows: args.rows,
    }))
    .await;
    Ok("File printed successfully.".to_string())
}
