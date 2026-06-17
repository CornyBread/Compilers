mod file_reader; 
use file_reader::FileReader;

fn main() {
    let ruta = "filereader_prueba.txt";

    let inicio = 1;
    let fin = 11;

    println!("Leyendo desde la línea {} hasta la {}", inicio, fin);
    
    match FileReader::read_lines_between(ruta, inicio, fin) {
        Ok(lineas) => {
            for (indice, texto) in lineas.iter().enumerate() {
                let numero_linea_real = inicio + indice;
                println!("Línea {}: {}", numero_linea_real, texto);
            }
        }
        Err(e) => println!("Error al leer el rango de líneas: {}", e),
    }
}