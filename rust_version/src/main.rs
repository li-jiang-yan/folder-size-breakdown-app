use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[allow(dead_code)]
struct FolderSize {
    path: PathBuf,
    bytes: u64,
}

fn folder_size(path: &Path, rows: &mut Vec<FolderSize>) -> io::Result<u64> {
    let mut total = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child_path = entry.path();
        let metadata = fs::metadata(&child_path)?;

        if metadata.is_dir() {
            // Recursively calculate this subfolder first.
            total += folder_size(&child_path, rows)?;
        } else if metadata.is_file() {
            total += metadata.len();
        }
    }

    // Record this folder after its contents have been counted.
    rows.push(FolderSize {
        path: path.to_path_buf(),
        bytes: total,
    });

    Ok(total)
}

fn main() {
    let start = Instant::now();
    let mut rows = Vec::new();
    let root: &Path = Path::new("C:\\GIT");
    println!("{:?}", folder_size(root, &mut rows));
    println!("Execution time: {:?} s", start.elapsed());
}
