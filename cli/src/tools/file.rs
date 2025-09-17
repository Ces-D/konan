use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

/// Reads a file line by line from a given path using an iterator.
/// This is memory-efficient as it doesn't load the whole file into memory.
/// Returns an iterator over the lines of the file.
pub fn read_file_lines<P: AsRef<Path>>(path: P) -> io::Result<io::Lines<BufReader<File>>> {
    let input_path = path.as_ref();
    let abs_path = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(input_path)
    };

    let file = File::open(abs_path)?;
    Ok(BufReader::new(file).lines())
}
