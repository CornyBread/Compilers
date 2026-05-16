use crate::util::Printable;

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

    pub fn add_child(&mut self, value: T) {
        self.children.push(TreeNode::new(value));
    }
}

pub struct Tree<T> {
    root: Option<TreeNode<T>>,
}

impl<T: PartialEq + Clone> Tree<T> {
    pub fn new() -> Self {
        Tree { root: None }
    }

    pub fn set_root(&mut self, value: T) {
        if self.root.is_none() {
            self.root = Some(TreeNode::new(value));
        }
    }

    pub fn find_node_mut(&mut self, target: &T) -> Option<&mut TreeNode<T>> {
        if let Some(ref mut root_node) = self.root {
            return Self::find_in_node(root_node, target);
        }
        None
    }

    fn find_in_node<'a>(current: &'a mut TreeNode<T>, target: &T) -> Option<&'a mut TreeNode<T>> {
        if current.value == *target {
            return Some(current); 
        }
        
        
        for child in &mut current.children {
            if let Some(found) = Self::find_in_node(child, target) {
                return Some(found); 
            }
        }
        None 
    }
}

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