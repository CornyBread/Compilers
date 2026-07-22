use crate::util::Printable;
use std::fmt;

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

#[derive(Debug, Clone)]
pub struct Simbolo {
    pub nombre: String,
    pub categoria: Categoria,
    pub tipo: String,
    pub ambito: String,
    pub parametros: Vec<(String, String)>,
    pub variadica: bool,
    pub inicializado: bool,
    pub usado: bool,
    pub linea: usize,
}

impl Simbolo {
    pub fn variable(
        nombre: impl Into<String>,
        tipo: impl Into<String>,
        ambito: impl Into<String>,
        inicializado: bool,
        linea: usize,
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
            linea,
        }
    }

    pub fn parametro(
        nombre: impl Into<String>,
        tipo: impl Into<String>,
        ambito: impl Into<String>,
        linea: usize,
    ) -> Self {
        Simbolo {
            nombre: nombre.into(),
            categoria: Categoria::Parametro,
            tipo: tipo.into(),
            ambito: ambito.into(),
            parametros: Vec::new(),
            variadica: false,
            inicializado: true,
            usado: false,
            linea,
        }
    }

    pub fn funcion(
        nombre: impl Into<String>,
        retorno: impl Into<String>,
        ambito: impl Into<String>,
        parametros: Vec<(String, String)>,
        linea: usize,
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
            linea,
        }
    }
}

pub struct TablaSimbolos {
    simbolos: Vec<Simbolo>,
}

impl TablaSimbolos {
    pub fn new() -> Self {
        TablaSimbolos {
            simbolos: Vec::new(),
        }
    }

    pub fn existe(&self, nombre: &str, ambito: &str) -> bool {
        self.simbolos
            .iter()
            .any(|s| s.nombre == nombre && s.ambito == ambito)
    }

    pub fn declarar(&mut self, simbolo: Simbolo) -> bool {
        if self.existe(&simbolo.nombre, &simbolo.ambito) {
            return false;
        }
        self.simbolos.push(simbolo);
        true
    }

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
