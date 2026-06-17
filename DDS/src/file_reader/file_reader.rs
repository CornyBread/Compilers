use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};

pub struct FileReader;

#[allow(dead_code)]
impl FileReader {
    pub fn read_all(file_path: &str) -> io::Result<String> {
        fs::read_to_string(file_path)
    }

    pub fn read_lines_between(file_path: &str, start: usize, end: usize) -> io::Result<Vec<String>> {
        if start == 0 || start > end {
            return Ok(Vec::new());
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut lines_result = Vec::new();

        let cantidad_a_tomar = end - start + 1;

        for line in reader.lines().skip(start - 1).take(cantidad_a_tomar) {
            lines_result.push(line?);
        }

        Ok(lines_result)
    }
}
