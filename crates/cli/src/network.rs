use anyhow::{Context, Result};
use cli_shared::RemoteFile;
use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::{Path, PathBuf};

pub struct Network {
    session: Session,
}
impl Network {
    pub fn new() -> Result<Self> {
        let remote_addr = std::env::var("KONAN_PI_REMOTE_HOST")
            .with_context(|| "Missing raspberry pi host addr")?;
        let remote_username = std::env::var("KONAN_PI_REMOTE_USERNAME")
            .with_context(|| "Missing raspberry pi username")?;
        let remote_password = std::env::var("KONAN_PI_REMOTE_PASSSWORD")
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

    pub fn execute_command(&mut self, command: String) -> Result<String> {
        let mut channel = self.session.channel_session().unwrap();
        channel
            .exec(&command)
            .with_context(|| "Unable to execute remote command")?;
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        channel.close()?;
        Ok(s)
    }

    fn prepare_file(p: &PathBuf) -> Result<(RemoteFile, i32, u64)> {
        // Check the path exists and is a file
        if !p.exists() {
            anyhow::bail!("File does not exist: {}", p.display());
        }
        if !p.is_file() {
            anyhow::bail!("Path is not a file: {}", p.display());
        }

        let remote_file = match p.extension() {
            Some(extension) => match extension.to_str() {
                Some("md") => RemoteFile::Markdown,
                // Validate extension is .md or .txt
                Some("txt") => RemoteFile::Text,
                _ => anyhow::bail!(
                    "File must be a markdown (.md) or text (.txt) file, got: {:?}",
                    extension
                ),
            },
            None => RemoteFile::Text,
        };

        // Unix file mode: 0o644 = owner read/write, group/others read-only
        let mode: i32 = 0o644;

        // File size must be known ahead of time
        let size: u64 = p
            .metadata()
            .with_context(|| format!("Failed to read metadata for: {}", p.display()))?
            .len();

        Ok((remote_file, mode, size))
    }

    pub fn upload_file(&mut self, path: &PathBuf) -> Result<RemoteFile> {
        let (rf, mode, size) = Self::prepare_file(&path)?;
        let mut remote_file = self
            .session
            .scp_send(Path::new(&rf.file_name()), mode, size, None)
            .with_context(|| "Failed to send {} over secure copy protocol")?;
        let local_file = std::fs::read(path)?;
        remote_file.write_all(&local_file)?;
        remote_file.send_eof()?;
        remote_file.wait_eof()?;
        remote_file.close()?;
        remote_file.wait_close()?;
        Ok(rf)
    }
}
