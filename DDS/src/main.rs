// Declaramos los módulos (nuestros archivos separados) que componen el proyecto.
// mod util;
// mod stack;
// mod queue;
// mod map;
// mod tree;
mod file_reader;
mod logger;
mod lexer;

// Traemos las estructuras a este archivo para poder instanciarlas.
// use util::Printable;
// use stack::Stack;
// use queue::Queue;
// use map::Map;
// use tree::Tree;
use lexer::Lexer;


fn main() {
    // println!("--- Probando Pila ---");
    // let mut my_stack = Stack::new();
    
    // // Apilamos elementos. El último en entrar (30) será el primero en salir.
    // my_stack.push(10);
    // my_stack.push(20);
    // my_stack.push(30);
    // my_stack.print_structure();
    
    // // Sacamos el elemento del tope. Lo imprimimos con {:?} porque es de tipo Option.
    // let popped_item = my_stack.pop();
    // println!("Elemento desapilado: {:?}", popped_item);
    // my_stack.print_structure();

    // println!("\n--- Probando Cola ---");
    // let mut my_queue = Queue::new();
    
    // // Encolamos elementos. El primero en entrar (10) será el primero en salir.
    // my_queue.enqueue(10);
    // my_queue.enqueue(20);
    // my_queue.enqueue(30);
    // my_queue.print_structure();
    
    // // Sacamos el primer elemento de la fila.
    // let dequeued_item = my_queue.dequeue();
    // println!("Elemento desencolado: {:?}", dequeued_item);
    // my_queue.print_structure();

    // println!("\n--- Probando Mapa ---");
    // let mut my_map = Map::new();

    // my_map.insert("uno", 1);
    // my_map.insert("dos", 2);
    // my_map.insert("tres", 3);
    // my_map.print_structure();

    // let value = my_map.get(&"dos");
    // println!("Valor encontrado para 'dos': {:?}", value);

    // println!("\n--- Probando Arbol ---");
    
    // let mut num_tree: Tree<i32> = Tree::new();
    // num_tree.set_root(100);

    // if let Some(raiz) = num_tree.find_node_mut(&100) {
    //     raiz.add_child(50);
    //     raiz.add_child(150);
    //     raiz.add_child(200);
    // }

    // if let Some(nodo_50) = num_tree.find_node_mut(&50) {
    //     nodo_50.add_child(10);
    //     nodo_50.add_child(20);
    //     nodo_50.add_child(30);
    // }

    // if let Some(nodo_150) = num_tree.find_node_mut(&150) {
    //     nodo_150.add_child(125);
    // }

    // if let Some(nodo_200) = num_tree.find_node_mut(&200) {
    //     nodo_200.add_child(300);
    // }
    // if let Some(nodo_300) = num_tree.find_node_mut(&300) {
    //     nodo_300.add_child(400);
    // }

    // // Imprimimos la estructura
    // num_tree.print_structure();

    println!("\n--- Analizador Léxico (Python) ---");

    let ruta = "ejemplo.py";
    match Lexer::from_file(ruta) {
        Ok(mut lexer) => {
            // 2 y 3: extrae/clasifica y devuelve la lista de tokens.
            let tokens = lexer.tokenize();

            println!("Tokens encontrados en '{}': {}", ruta, tokens.len());
            for token in &tokens {
                println!("{}", token);
            }

            // Reporte de errores léxicos a través del Logger.
            let errores = lexer.logger().entries();
            if errores.is_empty() {
                println!("\nSin errores léxicos.");
            } else {
                println!("\nErrores léxicos ({}):", errores.len());
                for entry in errores {
                    println!("{}", entry);
                }
            }
        }
        Err(e) => eprintln!("No se pudo leer '{}': {}", ruta, e),
    }
}