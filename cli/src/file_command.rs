use anyhow::bail;
use clap::Parser;
use log::{info, trace};
use rongta::TextDecoration;
use std::{
    ffi::OsStr,
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

    #[clap(short, long, help = "Number of rows per page (cuts after each page)")]
    rows: Option<usize>,
}

pub async fn handle_file_command(args: FileArgs, lines: Option<u32>) -> anyhow::Result<()> {
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

    let mut builder = rongta::PrintBuilder::new(false);
    let file_content = read_file_lines(&args.path)?;

    for line in file_content {
        let line = line?;
        trace!("Reading line: {}", line);
        builder.set_justify_content(rongta::Justify::Left);
        builder.set_text_decoration(TextDecoration {
            bold: true,
            ..Default::default()
        });
        builder.add_content(&line)?;
        builder.new_line();
    }

    // If args.rows is specified, use that; otherwise use the global lines parameter
    let pagination = args.rows.map(|r| r as u32).or(lines);
    builder.print(pagination)?;

    Ok(())
}
