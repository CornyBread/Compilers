// Declaramos los módulos (nuestros archivos separados) que componen el proyecto.
mod util;
// mod stack;
// mod queue;
// mod map;
mod tree;
mod file_reader;
mod logger;
mod lexer;
mod parser;
mod semantic;
mod transpiler;

// Traemos las estructuras a este archivo para poder instanciarlas.
use util::Printable;
use lexer::Lexer;
use logger::LogLevel;
use parser::Parser;
use semantic::AnalizadorSemantico;
use transpiler::Transpilador;

fn main() {
    // Se puede pasar otro archivo por argumento: `cargo run -- otro.py`.
    let ruta = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "programa.py".to_string());
    let ruta = ruta.as_str();

    // 1. Análisis léxico: del archivo fuente obtenemos la lista de tokens.
    let mut lexer = match Lexer::from_file(ruta) {
        Ok(lexer) => lexer,
        Err(e) => {
            eprintln!("No se pudo leer '{}': {}", ruta, e);
            return;
        }
    };
    let tokens = lexer.tokenize();

    println!("--- Analizador Léxico ---");
    println!("Tokens encontrados en '{}': {}", ruta, tokens.len());

    // Si el léxico tiene errores, no tiene sentido continuar al sintáctico.
    let errores_lexicos = lexer.logger().entries();
    if !errores_lexicos.is_empty() {
        println!("\nErrores léxicos ({}):", errores_lexicos.len());
        for entry in errores_lexicos {
            println!("{}", entry);
        }
        println!("\nSe detiene el proceso por errores léxicos.");
        return;
    }
    println!("Sin errores léxicos.");

    // 2. Análisis sintáctico: los tokens son la entrada del parser.
    println!("\n--- Analizador Sintáctico ---");
    let mut parser = Parser::new(tokens);
    match parser.analizar() {
        // Éxito: se imprime el árbol sintáctico y se pasa al semántico.
        Some(arbol) => {
            println!("Árbol sintáctico generado:\n");
            arbol.print_structure();

            // 3. Análisis semántico: se recorre el árbol UNA vez (izquierda a
            // derecha) llenando la Tabla de Símbolos con cada declaración.
            println!("\n--- Analizador Semántico ---");
            let mut semantico = AnalizadorSemantico::new();
            semantico.analizar(&arbol);

            // Las advertencias (dead code, etc.) no invalidan el programa.
            let advertencias: Vec<_> = semantico
                .logger()
                .entries()
                .iter()
                .filter(|e| e.level == LogLevel::Warn)
                .collect();
            if !advertencias.is_empty() {
                println!("\nAdvertencias ({}):", advertencias.len());
                for entry in &advertencias {
                    println!("{}", entry);
                }
            }

            // Los errores sí: se reportan todos y no se muestra la tabla.
            let errores: Vec<_> = semantico
                .logger()
                .entries()
                .iter()
                .filter(|e| e.level == LogLevel::Error)
                .collect();
            if errores.is_empty() {
                println!("\nSin errores semánticos.");
                // La tabla queda lista para la fase de ejecución: ya no hará
                // falta recorrer el árbol para verificar declaraciones.
                println!();
                semantico.tabla().print_structure();

                // 4. Transpilación a Java: como no hubo errores, recorremos el
                // árbol otra vez para emitir el código Java equivalente.
                println!("\n--- Transpilador a Java ---");
                let clase = nombre_de_clase(ruta);
                let java = Transpilador::new(&clase).transpilar(&arbol);
                let salida = format!("{}.java", clase);
                match std::fs::write(&salida, java) {
                    Ok(_) => println!("Código Java generado en '{}'.", salida),
                    Err(e) => eprintln!("No se pudo escribir '{}': {}", salida, e),
                }
            } else {
                println!("\nPrograma inválido por errores semánticos ({}):", errores.len());
                for entry in &errores {
                    println!("{}", entry);
                }
            }
        }

        None => {
            let errores = parser.logger().entries();
            println!("Se canceló el árbol por errores sintácticos ({}):", errores.len());
            for entry in errores {
                println!("{}", entry);
            }
        }
    }
}

// Deriva el nombre de la clase Java a partir del archivo fuente: en Java la
// clase pública debe llamarse igual que el archivo (`Programa.java`).
fn nombre_de_clase(ruta: &str) -> String {
    let stem = std::path::Path::new(ruta)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Programa");
    let mut chars = stem.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => "Programa".to_string(),
    }
}
