use crate::clap_enum::AllowedCommand;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(help = "The file path")]
    pub path: PathBuf,
    #[clap(long, help = "Number of rows per page (cuts after each page)")]
    pub rows: Option<u32>,
    #[clap(long, help = "A cli command whose output is piped to file")]
    pub prehook_command: Option<AllowedCommand>,
    #[clap(long, help = "Dynamic cli command arg")]
    pub prehook_command_args: Option<String>,
}
