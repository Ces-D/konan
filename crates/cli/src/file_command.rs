use crate::{command_builder::PiCommandBuilder, network::Network};
use anyhow::bail;
pub use cli_shared::file_command::FileArgs;

pub async fn handle_file_command(args: FileArgs, cut: bool) -> anyhow::Result<()> {
    let mut conn = Network::new()?;
    match conn.upload_file(&args.path, true) {
        Ok(remote_file) => {
            let cmd = PiCommandBuilder::new("file")
                .positional(&remote_file)
                .named("rows", args.rows)
                .flag("no-cut", !cut)
                .named("prehook-command", args.prehook_command)
                .named("prehook-command-args", args.prehook_command_args);
            conn.execute_command(cmd)
        }
        Err(e) => {
            log::error!("Failed to upload file to remote host: {:?}", e);
            bail!("Failed to upload printable file: {:?}", args.path.display())
        }
    }
}
