use clap::Parser;
use cli_shared::RemoteFile;

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(short, long, help = "Remote file to print")]
    file: RemoteFile,
    #[clap(short, long, help = "Number of rows per page (cuts after each page)")]
    rows: Option<u32>,
}

pub async fn handle_file_command(args: FileArgs, cut: bool) -> anyhow::Result<()> {
    crate::print_ops::print_file(args.file, cut, args.rows)
}
