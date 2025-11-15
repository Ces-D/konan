use anyhow::bail;
use clap::Parser;
use log::info;
use log::trace;
use std::ffi::OsStr;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
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

pub fn read_file_complete<P: AsRef<Path>>(path: P) -> io::Result<String> {
    trace!("Reading file complete");
    let input_path = path.as_ref();
    let abs_path = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(input_path)
    };
    let mut file = File::open(abs_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

#[derive(Debug, Parser)]
pub struct FileArgs {
    path: std::path::PathBuf,
    #[clap(help = "The file path")]
    #[clap(
        short = 'i',
        long = "as-is",
        help = "A flag identifying that this should print without formatting",
        default_value_t = false
    )]
    as_is: bool,
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
    let mut printer = rongta::establish_rongta_printer()?;
    match args.as_is {
        true => {
            let file_content = read_file_complete(&args.path)?;
            if no_cut {
                printer.writeln(&file_content)?.print()?;
            } else {
                printer.writeln(&file_content)?.print_cut()?;
            }
        }
        false => {
            let mut print_builder = rongta::PrintBuilder::default();
            print_builder.cut = !no_cut;
            let file_content = read_file_lines(&args.path)?;
            for line in file_content {
                let line = line?;
                print_builder.add_content(&line, rongta::TextSize::Medium, false, false)?;
            }
            print_builder.print(printer)?;
        }
    };
    Ok(())
}
