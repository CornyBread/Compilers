// Representación del Árbol Sintáctico (AST).
//
// Reutilizamos el árbol genérico ya existente (`TreeNode<T>`): cada nodo del
// árbol sintáctico es simplemente un nodo etiquetado con un `String`. Así el
// árbol se puede imprimir con el mismo `print_structure` del módulo `tree`,
// tal como se dibujó en el pizarrón (root -> Función -> Cuerpo -> ...).

use crate::tree::tree::TreeNode;

/// Un nodo del árbol sintáctico: un nodo de árbol cuya etiqueta es texto.
pub type Nodo = TreeNode<String>;

/// Crea un nodo hoja con la etiqueta indicada.
pub fn nodo(etiqueta: impl Into<String>) -> Nodo {
    TreeNode::new(etiqueta.into())
}

/// Crea un nodo con su etiqueta y una lista de hijos ya construidos.
pub fn nodo_con(etiqueta: impl Into<String>, hijos: Vec<Nodo>) -> Nodo {
    let mut n = TreeNode::new(etiqueta.into());
    for hijo in hijos {
        n.add_child_node(hijo);
    }
    n
}
