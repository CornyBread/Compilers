mod file_writer; 
use file_writer::FileWriter;

fn main() {
    let ruta_archivo = "filewriter_prueba.txt";

    println!("--- Probando Escritura y Append ---");
    let mensaje_inicial = "Línea 1: Este es el inicio del archivo.";
    if let Ok(_) = FileWriter::write_output(ruta_archivo, mensaje_inicial) {
        println!("1. Archivo creado/sobrescrito con éxito.");
    }
    let mensaje_extra = "Línea 2: Esto se agregó sin borrar la Línea 1.";
    match FileWriter::append_output(ruta_archivo, mensaje_extra) {
        Ok(_) => {
            println!("2.Éxito: Se agregó contenido extra a '{}'.", ruta_archivo);
        }
        Err(e) => {
            eprintln!("2.Error al agregar: {}", e);
        }
    }
    
    println!("\nPrueba terminada. Abre el archivo para ver los resultados.");
}