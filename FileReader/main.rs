mod file_reader; 
use file_reader::FileReader;

fn main() {
    let ruta = "filereader_prueba.txt";

    println!("--- PRUEBA 1: Leyendo todo---");
    match FileReader::read_all(ruta) {
        Ok(contenido) => println!("{}", contenido),
        Err(e) => println!("Error al leer todo: {}", e),
    }

    println!("\n--- PRUEBA 2: Leyendo solo 3 líneas ---");
    match FileReader::read_n_lines(ruta, 3) {
        Ok(lineas) => {
            for (numero, texto) in lineas.iter().enumerate() {
                println!("Línea {}: {}", numero + 1, texto);
            }
        }
        Err(e) => println!("Error al leer líneas: {}", e),
    }
}