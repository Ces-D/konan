use crate::{command_builder::PiCommandBuilder, network::Network};
use anyhow::bail;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(help = "The file path")]
    pub path: PathBuf,
    #[clap(long, help = "Number of rows per page (cuts after each page)")]
    pub rows: Option<u32>,
}

pub async fn handle_file_command(args: FileArgs, cut: bool) -> anyhow::Result<()> {
    let mut conn = Network::new()?;
    match conn.upload_file(&args.path) {
        Ok(remote_file) => {
            let cmd = PiCommandBuilder::new("file")
                .named_enum("file", Some(remote_file))
                .named("rows", args.rows)
                .flag("no-cut", !cut);
            conn.execute_command(cmd)
        }
        Err(e) => {
            log::error!("Failed to upload file to remote host: {:?}", e);
            bail!("Failed to upload printable file: {:?}", args.path.display())
        }
    }
}
