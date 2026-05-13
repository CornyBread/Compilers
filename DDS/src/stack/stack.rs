use crate::util::Printable;

enum Link<T> {
    Empty,
    Next(Box<Node<T>>),
}

// Estructura de un nodo individual en la memoria.
struct Node<T> {
    value: T,
    next: Link<T>, // Apunta al nodo que está debajo en la pila.
}

// Estructura principal de la Pila (Stack - LIFO: Último en entrar, primero en salir).
pub struct Stack<T> {
    head: Link<T>, // El tope de la pila.
}

impl<T> Stack<T> {
    // Constructor para iniciar una pila completamente vacía.
    pub fn new() -> Self {
        Stack {
            head: Link::Empty,
        }
    }

    // Metodo para agregar un elemento al tope de la pila (Push / Apilar).
    pub fn push(&mut self, value: T) {
        // Box::new asigna la memoria dinámicamente en el Heap.
        let new_node = Box::new(Node {
            value,
            next: std::mem::replace(&mut self.head, Link::Empty),
        });
        self.head = Link::Next(new_node);
    }

    // Metodo para sacar el elemento del tope (Pop / Desapilar).
    pub fn pop(&mut self) -> Option<T> {
        match std::mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None, // Si está vacía, devolvemos None (equivalente seguro a null).
            Link::Next(node) => {
                // Si hay un nodo, la nueva cabeza será el nodo que estaba debajo.
                self.head = node.next;
                Some(node.value) // Devolvemos el valor envuelto en Some().
            }
        }
    }
}

// Implementamos el Trait Printable para el Stack.
impl<T: std::fmt::Display> Printable for Stack<T> {
    fn print_structure(&self) {
        print!("Pila (Tope -> Fondo): "); 
        let mut current = &self.head;
        // Iteramos mientras el enlace actual tenga un nodo 'Next'
        while let Link::Next(node) = current {
            print!("[{}] ", node.value);
            current = &node.next; // Avanzamos al siguiente nodo
        }
        println!();
    }
}