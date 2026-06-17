use crate::util::Printable;
use std::collections::VecDeque;

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

impl<T: std::fmt::Display> Printable for Queue<T> {
    fn print_structure(&self) {
        print!("Cola (Frente -> Final): ");
        for element in &self.elements {
            print!("[{}] ", element);
        }
        println!();
    }
}