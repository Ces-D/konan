use crate::command_builder::PiCommandBuilder;
use anyhow::{Context, Result};
use ssh2::Session;
use std::{
    io::prelude::*,
    net::TcpStream,
    path::{Path, PathBuf},
};

pub struct Network {
    session: Session,
}
impl Network {
    pub fn new() -> Result<Self> {
        // TODO: connect via ssh
        let remote_addr = std::env::var("KONAN_PI_REMOTE_HOST")
            .with_context(|| "Missing raspberry pi host addr")?;
        let remote_username = std::env::var("KONAN_PI_REMOTE_USERNAME")
            .with_context(|| "Missing raspberry pi username")?;
        let remote_password = std::env::var("KONAN_PI_REMOTE_PASSWORD")
            .with_context(|| "Missing raspberry pi password")?;
        // 1. Connect to the Pi
        let tcp = TcpStream::connect(remote_addr)?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;
        // 2. Authenticate
        sess.userauth_password(&remote_username, &remote_password)
            .with_context(|| "Failed to authenticate to remote raspberry pi")?;
        Ok(Self { session: sess })
    }

    pub fn execute_command(&mut self, command: PiCommandBuilder) -> Result<()> {
        let command = command.build();
        let mut channel = self.session.channel_session().unwrap();
        channel
            .exec(&command)
            .with_context(|| "Unable to execute remote command")?;
        log::info!("Command executed");

        let mut stdout = String::new();
        channel.read_to_string(&mut stdout).unwrap();

        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr).unwrap();

        channel.close()?;
        channel.wait_close()?;

        if !stdout.is_empty() {
            println!("{}", stdout);
        }

        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }

        let exit_status = channel.exit_status()?;
        if exit_status != 0 {
            anyhow::bail!("Remote command exited with status {}", exit_status);
        }

        Ok(())
    }

    fn prepare_file(p: &Path, replace_file_name: bool) -> Result<(String, i32, u64)> {
        // Check the path exists and is a file
        if !p.exists() {
            anyhow::bail!("File does not exist: {}", p.display());
        }
        if !p.is_file() {
            anyhow::bail!("Path is not a file: {}", p.display());
        }

        let extension: SupportedExtension = match p.extension() {
            Some(extension) => match extension.to_str() {
                Some("md") => SupportedExtension::Md,
                Some("txt") => SupportedExtension::Txt,
                _ => anyhow::bail!(
                    "File must be a markdown (.md) or text (.txt) file, got: {:?}",
                    extension
                ),
            },
            None => {
                anyhow::bail!("File must be a markdown (.md) or text (.txt) file")
            }
        };

        let file_name = match replace_file_name {
            true => match extension {
                SupportedExtension::Txt => "konan_print.txt".to_string(),
                SupportedExtension::Md => "konan_print.md".to_string(),
            },
            false => p
                .file_name()
                .context("Path has no file name")?
                .to_string_lossy()
                .into_owned(),
        };

        // Unix file mode: 0o644 = owner read/write, group/others read-only
        let mode: i32 = 0o644;

        // File size must be known ahead of time
        let size: u64 = p
            .metadata()
            .with_context(|| format!("Failed to read metadata for: {}", p.display()))?
            .len();

        Ok((file_name, mode, size))
    }

    /// Build the remote path for a file in the printer files directory.
    /// Keep in sync with pi_cli/src/config.rs -> printer_files_dir_path
    fn remote_files_path(file_name: &str) -> String {
        format!(
            "{}/{}/{}",
            cli_shared::APPLICATION_STORAGE_DIR,
            "files",
            file_name
        )
    }

    pub fn upload_file(&mut self, path: &PathBuf, replace_file_name: bool) -> Result<String> {
        let (file_name, mode, size) = Self::prepare_file(path, replace_file_name)?;
        let remote_path = Self::remote_files_path(&file_name);
        self.scp_upload(path, &remote_path, mode, size)?;
        Ok(file_name)
    }

    fn scp_upload(&mut self, local: &Path, remote: &str, mode: i32, size: u64) -> Result<()> {
        let mut remote_file = self
            .session
            .scp_send(Path::new(remote), mode, size, None)
            .with_context(|| format!("Failed to send '{}' over secure copy protocol", remote))?;
        let local_file = std::fs::read(local)?;
        remote_file.write_all(&local_file)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(())
    }
}

enum SupportedExtension {
    Txt,
    Md,
}
