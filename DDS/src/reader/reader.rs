use std::fs;
use std::io;

// Estructura que actuará como nuestro Lector de Archivos
pub struct FileReader;

impl FileReader {
    // Método para leer todo el contenido de un archivo fuente.
    // Recibe la ruta del archivo y devuelve un Result con el texto adentro.
    pub fn read_source_code(file_path: &str) -> io::Result<String> {
        // read_to_string abre el archivo, lee el texto y lo cierra automáticamente
        fs::read_to_string(file_path)
    }
}