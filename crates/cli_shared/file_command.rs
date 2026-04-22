use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(help = "The file path")]
    pub path: PathBuf,
    #[clap(long, help = "Number of rows per page (cuts after each page)")]
    pub rows: Option<u32>,
}
