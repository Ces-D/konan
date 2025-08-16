use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Reads a file from a given path, whether it's relative or absolute.
/// Returns the file contents as a `String`.
pub fn read_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    // Convert to a PathBuf
    let input_path = path.as_ref();

    // Resolve to an absolute path
    let abs_path: PathBuf = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(input_path)
    };

    // Optional: Canonicalize to remove `..` and `.` segments
    let abs_path = abs_path.canonicalize()?;

    // Read the file contents
    fs::read_to_string(abs_path)
}
