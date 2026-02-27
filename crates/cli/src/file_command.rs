use crate::network::Network;
use anyhow::bail;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(help = "The file path")]
    path: PathBuf,
    #[clap(short, long, help = "Number of rows per page (cuts after each page)")]
    rows: Option<u32>,
}

pub async fn handle_file_command(args: FileArgs, cut: bool) -> anyhow::Result<()> {
    let mut conn = Network::new()?;
    match conn.upload_file(&args.path) {
        Ok(remote_file) => {
            let mut cmd = "konan file".to_string();
            if args.rows.is_some() {
                cmd.push_str(&format!(
                    " --rows {} --file {}",
                    args.rows.unwrap_or_default(),
                    remote_file.file_name(),
                ));
            }
            if !cut {
                cmd.push_str(" --no_cut");
            }
            match conn.execute_command(cmd) {
                Ok(output) => {
                    println!("{}", output);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to call konan file command: {:?}", e);
                    bail!("Failed to execute remote konan file command")
                }
            }
        }
        Err(e) => {
            log::error!("Failed to upload file to remote host: {:?}", e);
            bail!("Failed to upload printable file: {:?}", args.path.display())
        }
    }
}
