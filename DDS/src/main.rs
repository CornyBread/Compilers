// Declaramos los módulos (nuestros archivos separados) que componen el proyecto.
mod util;
mod stack;
mod queue;
mod map;
mod tree;
// Traemos las estructuras a este archivo para poder instanciarlas.
use util::Printable;
use stack::Stack;
use queue::Queue;
use map::Map;
use tree::Tree;

fn main() {
    println!("--- Probando Pila ---");
    let mut my_stack = Stack::new();
    
    // Apilamos elementos. El último en entrar (30) será el primero en salir.
    my_stack.push(10);
    my_stack.push(20);
    my_stack.push(30);
    my_stack.print_structure();
    
    // Sacamos el elemento del tope. Lo imprimimos con {:?} porque es de tipo Option.
    let popped_item = my_stack.pop();
    println!("Elemento desapilado: {:?}", popped_item);
    my_stack.print_structure();

    println!("\n--- Probando Cola ---");
    let mut my_queue = Queue::new();
    
    // Encolamos elementos. El primero en entrar (10) será el primero en salir.
    my_queue.enqueue(10);
    my_queue.enqueue(20);
    my_queue.enqueue(30);
    my_queue.print_structure();
    
    // Sacamos el primer elemento de la fila.
    let dequeued_item = my_queue.dequeue();
    println!("Elemento desencolado: {:?}", dequeued_item);
    my_queue.print_structure();

    println!("\n--- Probando Mapa ---");
    let mut my_map = Map::new();

    my_map.insert("uno", 1);
    my_map.insert("dos", 2);
    my_map.insert("tres", 3);
    my_map.print_structure();

    let value = my_map.get(&"dos");
    println!("Valor encontrado para 'dos': {:?}", value);

    println!("--- Probando Árbol General con Integros ---");
    
    // Instanciamos el árbol indicando que será de tipo i32
    let mut num_tree: Tree<i32> = Tree::new();

    // 1. Definimos la raíz
    num_tree.set_root(100);

    // 2. Insertamos hijos bajo la raíz (100)
    num_tree.insert_under(&100, 50);
    num_tree.insert_under(&100, 150);
    num_tree.insert_under(&100, 200);

    // 3. Insertamos nietos (bajo el 50)
    num_tree.insert_under(&50, 10);
    num_tree.insert_under(&50, 20);
    num_tree.insert_under(&50, 30);

    // 4. Insertamos bajo el 150
    num_tree.insert_under(&150, 125);

    // 5. Insertamos bajo el 200 (un nivel más profundo)
    num_tree.insert_under(&200, 300);
    num_tree.insert_under(&300, 400);

    // Imprimimos la estructura
    num_tree.print_structure();
}