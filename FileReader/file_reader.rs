use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};

pub struct FileReader; 

#[allow(dead_code)]
impl FileReader {
    pub fn read_all(file_path: &str) -> io::Result<String> {
        fs::read_to_string(file_path)
    }

    pub fn read_n_lines(file_path: &str, limit: usize) -> io::Result<Vec<String>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines_result = Vec::new();

        for line in reader.lines().take(limit) {
            lines_result.push(line?);
        }

        Ok(lines_result)
    }

}