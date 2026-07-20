// Tabla de Símbolos — Proyecto 4 (Analizador Semántico).
//
// Aquí quedan registradas TODAS las declaraciones del programa: variables,
// parámetros y funciones. La idea del pizarrón: cuando el programa se
// ejecute, el intérprete consulta esta tabla directamente y ya no necesita
// recorrer otra vez el árbol para verificar las declaraciones.

use crate::util::Printable;
use std::fmt;

/// Qué clase de símbolo es cada entrada de la tabla.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Categoria {
    Variable,
    Parametro,
    Funcion,
}

impl fmt::Display for Categoria {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let etiqueta = match self {
            Categoria::Variable => "Variable",
            Categoria::Parametro => "Parámetro",
            Categoria::Funcion => "Función",
        };
        write!(f, "{}", etiqueta)
    }
}

/// Una entrada de la tabla: una declaración del programa.
#[derive(Debug, Clone)]
pub struct Simbolo {
    pub nombre: String,
    pub categoria: Categoria,
    /// Tipo de la variable/parámetro, o tipo de retorno si es función.
    pub tipo: String,
    /// Ámbito donde vive: "global", "main", "main::if#1", ...
    pub ambito: String,
    /// Solo funciones: (nombre, tipo) de cada parámetro, en orden.
    pub parametros: Vec<(String, String)>,
    /// Solo funciones nativas como `print`: acepta cualquier cantidad de argumentos.
    pub variadica: bool,
    /// ¿Ya recibió un valor? (una declaración sin `=` queda sin inicializar).
    pub inicializado: bool,
    /// ¿El programa lo usa en algún punto? (sirve para avisar de dead code).
    pub usado: bool,
}

impl Simbolo {
    pub fn variable(
        nombre: impl Into<String>,
        tipo: impl Into<String>,
        ambito: impl Into<String>,
        inicializado: bool,
    ) -> Self {
        Simbolo {
            nombre: nombre.into(),
            categoria: Categoria::Variable,
            tipo: tipo.into(),
            ambito: ambito.into(),
            parametros: Vec::new(),
            variadica: false,
            inicializado,
            usado: false,
        }
    }

    pub fn parametro(
        nombre: impl Into<String>,
        tipo: impl Into<String>,
        ambito: impl Into<String>,
    ) -> Self {
        Simbolo {
            nombre: nombre.into(),
            categoria: Categoria::Parametro,
            tipo: tipo.into(),
            ambito: ambito.into(),
            parametros: Vec::new(),
            variadica: false,
            inicializado: true, // un parámetro siempre llega con valor
            usado: false,
        }
    }

    pub fn funcion(
        nombre: impl Into<String>,
        retorno: impl Into<String>,
        ambito: impl Into<String>,
        parametros: Vec<(String, String)>,
    ) -> Self {
        Simbolo {
            nombre: nombre.into(),
            categoria: Categoria::Funcion,
            tipo: retorno.into(),
            ambito: ambito.into(),
            parametros,
            variadica: false,
            inicializado: true,
            usado: false,
        }
    }
}

/// La tabla en sí: la lista de símbolos en el orden en que se declararon
/// (el mismo orden en que el árbol se recorre de izquierda a derecha).
pub struct TablaSimbolos {
    simbolos: Vec<Simbolo>,
}

impl TablaSimbolos {
    pub fn new() -> Self {
        TablaSimbolos {
            simbolos: Vec::new(),
        }
    }

    /// ¿Ya existe un símbolo con ese nombre en ese ámbito exacto?
    pub fn existe(&self, nombre: &str, ambito: &str) -> bool {
        self.simbolos
            .iter()
            .any(|s| s.nombre == nombre && s.ambito == ambito)
    }

    /// Registra una declaración. Devuelve `false` si el nombre ya existía en
    /// ese mismo ámbito -> caso "Redeclaraciones" del pizarrón.
    pub fn declarar(&mut self, simbolo: Simbolo) -> bool {
        if self.existe(&simbolo.nombre, &simbolo.ambito) {
            return false;
        }
        self.simbolos.push(simbolo);
        true
    }

    /// Busca un nombre recorriendo los ámbitos activos del más interno al más
    /// externo. Si el símbolo existe pero su ámbito ya no está activo, no se
    /// encuentra -> caso "Datos fuera de bloque".
    pub fn resolver_mut(&mut self, nombre: &str, ambitos: &[String]) -> Option<&mut Simbolo> {
        for ambito in ambitos.iter().rev() {
            let pos = self
                .simbolos
                .iter()
                .position(|s| s.nombre == nombre && &s.ambito == ambito);
            if let Some(pos) = pos {
                return self.simbolos.get_mut(pos);
            }
        }
        None
    }

    pub fn simbolos(&self) -> &[Simbolo] {
        &self.simbolos
    }
}

impl Printable for TablaSimbolos {
    fn print_structure(&self) {
        println!("Tabla de Símbolos:");
        if self.simbolos.is_empty() {
            println!("(vacía)");
            return;
        }

        // Cada fila: nombre | categoría | tipo | ámbito | detalle.
        let filas: Vec<[String; 5]> = self
            .simbolos
            .iter()
            .map(|s| {
                let detalle = match s.categoria {
                    Categoria::Funcion => {
                        if s.variadica {
                            "(argumentos variables)".to_string()
                        } else {
                            let params: Vec<String> = s
                                .parametros
                                .iter()
                                .map(|(n, t)| format!("{}: {}", n, t))
                                .collect();
                            format!("({})", params.join(", "))
                        }
                    }
                    _ => {
                        let init = if s.inicializado { "inicializado" } else { "sin inicializar" };
                        let uso = if s.usado { "usado" } else { "sin usar" };
                        format!("{}, {}", init, uso)
                    }
                };
                [
                    s.nombre.clone(),
                    s.categoria.to_string(),
                    s.tipo.clone(),
                    s.ambito.clone(),
                    detalle,
                ]
            })
            .collect();

        let encabezado = ["Nombre", "Categoría", "Tipo", "Ámbito", "Detalle"];

        // Ancho de cada columna: el máximo entre encabezado y todas las filas.
        let mut anchos: Vec<usize> = encabezado.iter().map(|t| t.chars().count()).collect();
        for fila in &filas {
            for (i, celda) in fila.iter().enumerate() {
                anchos[i] = anchos[i].max(celda.chars().count());
            }
        }

        let imprimir = |celdas: &[String]| {
            let linea: Vec<String> = celdas
                .iter()
                .enumerate()
                .map(|(i, c)| format!("{:<ancho$}", c, ancho = anchos[i]))
                .collect();
            println!("| {} |", linea.join(" | "));
        };

        let separador: Vec<String> = anchos.iter().map(|a| "-".repeat(*a)).collect();
        println!("+-{}-+", separador.join("-+-"));
        imprimir(&encabezado.map(String::from));
        println!("+-{}-+", separador.join("-+-"));
        for fila in &filas {
            imprimir(fila);
        }
        println!("+-{}-+", separador.join("-+-"));
    }
}
