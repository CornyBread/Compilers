use crate::util::Printable;
use std::collections::VecDeque;

// Estructura principal de la Cola (Queue - FIFO: Primero en entrar, primero en salir).
// En lugar de punteros complejos dobles, usamos VecDeque de la librería 
// estándar que ya maneja arreglos dinámicos circulares de forma muy eficiente.
pub struct Queue<T> {
    elements: VecDeque<T>,
}

impl<T> Queue<T> {
    // Constructor para iniciar una cola vacía.
    pub fn new() -> Self {
        Queue {
            elements: VecDeque::new(),
        }
    }

    // Método para agregar un elemento al final de la cola (Enqueue / Encolar).
    pub fn enqueue(&mut self, value: T) {
        self.elements.push_back(value);
    }

    // Método para sacar el elemento del frente de la cola (Dequeue / Desencolar).
    pub fn dequeue(&mut self) -> Option<T> {
        self.elements.pop_front() // pop_front ya nos devuelve un Option<T> por defecto.
    }
}

// Implementamos el Trait Printable para la Queue.
impl<T: std::fmt::Display> Printable for Queue<T> {
    fn print_structure(&self) {
        print!("Cola (Frente -> Final): ");
        // Iteramos sobre los elementos internos de la estructura estándar.
        for element in &self.elements {
            print!("[{}] ", element);
        }
        println!();
    }
}