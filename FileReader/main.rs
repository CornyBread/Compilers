mod file_reader; 
use file_reader::FileReader; 

fn main() {
    println!("\n--- Probando File Reader ---");
        
    let ruta = "filereader_prueba.txt";
    let resultado_lectura = FileReader::read_source_code(ruta);

    match resultado_lectura {
        Ok(contenido) => {
            println!("¡Archivo leído con éxito! Contenido:");
            println!("{}", contenido);
        }
        Err(e) => {
            println!("No se pudo leer el archivo. Error: {}", e);
        }
    }
}