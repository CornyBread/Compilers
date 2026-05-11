use crate::util::Printable;

// Un enlace puede estar vacío o apuntar al siguiente nodo con un smart pointer.
enum Link<K, V> {
    Empty,
    Next(Box<Node<K, V>>),
}

// Nodo interno de la lista enlazada del mapa.
struct Node<K, V> {
    key: K,
    value: V,
    next: Link<K, V>,
}

// Estructura principal del mapa dinámico.
pub struct Map<K, V> {
    head: Link<K, V>,
}

impl<K, V> Map<K, V> {
    // Crea un mapa vacío.
    pub fn new() -> Self {
        Map {
            head: Link::Empty,
        }
    }

    // Inserta un par clave-valor.
    // Si la clave ya existe, actualiza su valor.
    pub fn insert(&mut self, key: K, value: V)
    where
        K: PartialEq,
    {
        let mut current = &mut self.head;

        while let Link::Next(node) = current {
            if node.key == key {
                node.value = value;
                return;
            }
            current = &mut node.next;
        }

        let new_node = Box::new(Node {
            key,
            value,
            next: std::mem::replace(&mut self.head, Link::Empty),
        });
        self.head = Link::Next(new_node);
    }

    // Busca un valor por su clave.
    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: PartialEq,
    {
        let mut current = &self.head;

        while let Link::Next(node) = current {
            if &node.key == key {
                return Some(&node.value);
            }
            current = &node.next;
        }

        None
    }
}

// Implementamos Printable para poder mostrar el contenido del mapa.
impl<K, V> Printable for Map<K, V>
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    fn print_structure(&self) {
        print!("Mapa: ");
        let mut current = &self.head;

        while let Link::Next(node) = current {
            print!("[{} => {}] ", node.key, node.value);
            current = &node.next;
        }

        println!();
    }
}