use std::fs;
use std::io;
use std::fs::OpenOptions;
use std::io::Write;

pub struct FileWriter;

impl FileWriter {
    pub fn write_output(file_path: &str, content: &str) -> io::Result<()> {
        let content_with_newline = format!("{}\n", content);
        fs::write(file_path, content_with_newline)
    }
    pub fn append_output(file_path: &str, content: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;
        writeln!(file, "{}", content)
    }
}