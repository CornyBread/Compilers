use crate::util::Printable;

// Estructura de un Nodo para un árbol de n-hijos.
// Usamos Vec para almacenar los hijos, que es la forma estándar en Rust
// para manejar estructuras dinámicas de ramificación múltiple.
pub struct TreeNode<T> {
    pub value: T,
    pub children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    pub fn new(value: T) -> Self {
        TreeNode {
            value,
            children: Vec::new(),
        }
    }
}

// Estructura principal del Árbol General.
pub struct Tree<T> {
    root: Option<TreeNode<T>>,
}

impl<T: PartialEq + Clone> Tree<T> {
    pub fn new() -> Self {
        Tree { root: None }
    }

    // Establece la raíz del árbol si está vacío.
    pub fn set_root(&mut self, value: T) {
        if self.root.is_none() {
            self.root = Some(TreeNode::new(value));
        }
    }

    // Busca un nodo con el valor 'parent_value' y le agrega un hijo.
    // Esta es una interacción básica de árboles generales.
    pub fn insert_under(&mut self, parent_value: &T, new_value: T) -> bool {
        if let Some(ref mut root_node) = self.root {
            return Self::find_and_insert(root_node, parent_value, new_value);
        }
        false
    }

    fn find_and_insert(current: &mut TreeNode<T>, target: &T, new_value: T) -> bool {
    if current.value == *target {
        current.children.push(TreeNode::new(new_value));
        return true;
    }
    
    // Aquí está el cambio
    for child in &mut current.children {
        // Usamos .clone() porque no sabemos en cuál rama se quedará el valor
        if Self::find_and_insert(child, target, new_value.clone()) { 
            return true;
        }
    }
    false
}
}

// Implementación de Printable con formato visual de árbol.
impl<T: std::fmt::Display> Printable for Tree<T> {
    fn print_structure(&self) {
        println!("Estructura del Árbol:");
        if let Some(ref node) = self.root {
            Self::print_recursive(node, "", true);
        } else {
            println!("(Árbol vacío)");
        }
    }
}

impl<T: std::fmt::Display> Tree<T> {
    // Método auxiliar para imprimir la forma del árbol con prefijos visuales.
    fn print_recursive(node: &TreeNode<T>, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        println!("{}{}{}", prefix, connector, node.value);

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        
        for (i, child) in node.children.iter().enumerate() {
            let last_child = i == node.children.len() - 1;
            Self::print_recursive(child, &new_prefix, last_child);
        }
    }
}