// Declaramos los módulos (nuestros archivos separados) que componen el proyecto.
mod util;
mod stack;
mod queue;
mod map;

// Traemos las estructuras a este archivo para poder instanciarlas.
use util::Printable;
use stack::Stack;
use queue::Queue;
use map::Map;

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
}