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

// Traemos las estructuras a este archivo para poder instanciarlas.
use util::Printable;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let ruta = "programa.py";

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
        // Éxito: se imprime el árbol sintáctico.
        Some(arbol) => {
            println!("Árbol sintáctico generado:\n");
            arbol.print_structure();
        }
        // Regla del proyecto: si hay errores, se cancela el árbol y se
        // imprime el origen (línea y columna) de cada error.
        None => {
            let errores = parser.logger().entries();
            println!("Se canceló el árbol por errores sintácticos ({}):", errores.len());
            for entry in errores {
                println!("{}", entry);
            }
        }
    }
}
