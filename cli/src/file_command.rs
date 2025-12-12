use anyhow::bail;
use clap::Parser;
use log::info;
use log::trace;
use rongta::TextDecoration;
use std::ffi::OsStr;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

/// Reads a file line by line from a given path using an iterator.
/// This is memory-efficient as it doesn't load the whole file into memory.
/// Returns an iterator over the lines of the file.
pub fn read_file_lines<P: AsRef<Path>>(path: P) -> io::Result<io::Lines<BufReader<File>>> {
    trace!("Reading file lines");
    let input_path = path.as_ref();
    let abs_path = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(input_path)
    };

    let file = File::open(abs_path)?;
    Ok(BufReader::new(file).lines())
}

#[derive(Debug, Parser)]
pub struct FileArgs {
    #[clap(help = "The file path")]
    path: std::path::PathBuf,
}

pub async fn handle_file_command(args: FileArgs, no_cut: bool) -> anyhow::Result<()> {
    if !args.path.exists() {
        bail!("Path does not exist: {}", args.path.display());
    }
    if !args.path.is_file() {
        bail!("Path is not a file: {}", args.path.display());
    }
    let extension = args.path.extension().unwrap_or_else(|| OsStr::new("txt"));
    if extension == "md" {
        info!("Future feature will pretty print markdown files");
    }
    let mut builder = rongta::PrintBuilder::new(!no_cut);
    let file_content = read_file_lines(&args.path)?;
    for line in file_content {
        let line = line?;
        trace!("Reading line: {}", line);
        builder.set_justify_content(rongta::Justify::Left);
        builder.set_text_decoration(TextDecoration::default());
        builder.add_content(&line)?;
        builder.new_line();
    }
    builder.print()?;

    Ok(())
}
