use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn write_to_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(false)
        .open(path)?;

    file.write_all(content.as_bytes())
}

pub fn append_to_file(path: &Path, content: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new().append(true).open(path)?;

    file.write_all(content.as_bytes())
}
