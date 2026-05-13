use std::fs;
use std::io;

// Cambiamos el nombre a PascalCase (convención de Rust)
pub struct FileReader; 

impl FileReader {
    pub fn read_source_code(file_path: &str) -> io::Result<String> {
        fs::read_to_string(file_path)
    }
}