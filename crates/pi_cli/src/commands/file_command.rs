use crate::shared::driver;
use blueprint::interpreter::{markdown, text};
use clap::{Parser, ValueEnum};
use cli_shared::RemoteFile;
use rongta::RongtaPrinter;

#[derive(Debug, Clone, ValueEnum)]
pub enum FileType {
    Markdown,
    Text,
}

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(short, long, help = "Remote file to print")]
    file: RemoteFile,
    #[clap(short, long, help = "Number of rows per page (cuts after each page)")]
    rows: Option<u32>,
}

pub async fn handle_file_command(args: FileArgs, cut: bool) -> anyhow::Result<()> {
    match args.file {
        RemoteFile::Markdown => {
            let mut interpeter = markdown::MarkdownInterpreter::new(RongtaPrinter::new(cut));
            let file_content = std::fs::read_to_string(RemoteFile::Markdown.file_name())?;
            interpeter.print(&file_content, args.rows, driver())
        }
        RemoteFile::Text => {
            let mut interpreter = text::TextInterpreter::new(RongtaPrinter::new(cut));
            let file_content = std::fs::read_to_string(RemoteFile::Text.file_name())?;
            interpreter.print(&file_content, args.rows, driver())
        }
    }
}
