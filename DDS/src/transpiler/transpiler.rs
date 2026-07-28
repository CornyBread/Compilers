// Transpilador: convierte el AST (Tree<String>) de nuestro semi-Python a
// código Java. Corre al final, cuando el semántico ya validó el programa.
// Recorre el mismo árbol una sola vez emitiendo código (los pasos del
// compilador "al revés"), y se limita a lo que el parser/semántico soportan.

use std::collections::HashSet;

use crate::parser::ast::Nodo;
use crate::tree::Tree;

pub struct Transpilador {
    clase: String,
    salida: String,
    nivel: usize,
    // Nombres ya declarados: los globales persisten; los locales se reinician
    // en cada función. Sirven para decidir si una asignación es la primera
    // (declaración implícita -> `var`) o una reasignación.
    globales: HashSet<String>,
    locales: HashSet<String>,
}

impl Transpilador {
    pub fn new(clase: impl Into<String>) -> Self {
        Transpilador {
            clase: clase.into(),
            salida: String::new(),
            nivel: 0,
            globales: HashSet::new(),
            locales: HashSet::new(),
        }
    }

    /// Punto de entrada: devuelve el código Java completo como texto.
    pub fn transpilar(&mut self, arbol: &Tree<String>) -> String {
        let raiz = match arbol.root() {
            Some(r) => r,
            None => return String::new(),
        };

        self.linea(&format!("public class {} {{", self.clase));
        self.nivel += 1;

        for sentencia in &raiz.children {
            match sentencia.value.as_str() {
                // Las variables globales pasan a campos estáticos.
                "Declaración" => self.declaracion(sentencia, true),
                "Función" => {
                    self.salida.push('\n');
                    self.funcion(sentencia);
                }
                // La llamada suelta a main() no se traduce: en Java el arranque
                // ya lo da el propio método main.
                _ => {}
            }
        }

        self.nivel -= 1;
        self.linea("}");
        self.salida.clone()
    }

    // --- Definiciones ----------------------------------------------------

    fn funcion(&mut self, nodo: &Nodo) {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();
        let retorno = Self::valor_de(nodo, "retorno").unwrap_or_else(|| "Void".to_string());

        self.locales.clear();

        if nombre == "main" {
            self.linea("public static void main(String[] args) {");
        } else {
            let params = Self::hijo(nodo, "Parámetros")
                .map(|p| {
                    p.children
                        .iter()
                        .map(|par| match par.value.split_once(": ") {
                            Some((n, t)) => {
                                self.locales.insert(n.to_string());
                                format!("{} {}", Self::tipo_java(t), n)
                            }
                            None => par.value.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            self.linea(&format!(
                "public static {} {}({}) {{",
                Self::tipo_java(&retorno),
                nombre,
                params
            ));
        }

        self.nivel += 1;
        if let Some(cuerpo) = Self::hijo(nodo, "Cuerpo") {
            self.cuerpo(cuerpo);
        }
        self.nivel -= 1;
        self.linea("}");
    }

    // --- Sentencias ------------------------------------------------------

    fn cuerpo(&mut self, nodo: &Nodo) {
        for sentencia in &nodo.children {
            self.sentencia(sentencia);
        }
    }

    fn sentencia(&mut self, nodo: &Nodo) {
        match nodo.value.as_str() {
            "Declaración" => self.declaracion(nodo, false),
            "Asignación" => self.asignacion(nodo),
            "Expresión" => {
                let expr = self.expr(nodo);
                self.linea(&format!("{};", expr));
            }
            "if" => self.condicional(nodo),
            "while" => self.ciclo_while(nodo),
            "for" => self.ciclo_for(nodo),
            "return" => self.retorno(nodo),
            _ => {}
        }
    }

    fn declaracion(&mut self, nodo: &Nodo, estatico: bool) {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();
        let tipo = Self::tipo_java(&Self::valor_de(nodo, "tipo").unwrap_or_default());
        let prefijo = if estatico {
            self.globales.insert(nombre.clone());
            "static "
        } else {
            self.locales.insert(nombre.clone());
            ""
        };

        match Self::hijo(nodo, "valor") {
            Some(valor) => {
                let expr = self.expr(valor);
                self.linea(&format!("{}{} {} = {};", prefijo, tipo, nombre, expr));
            }
            None => self.linea(&format!("{}{} {};", prefijo, tipo, nombre)),
        }
    }

    fn asignacion(&mut self, nodo: &Nodo) {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();
        let operador = Self::valor_de(nodo, "operador").unwrap_or_else(|| "=".to_string());
        let valor = Self::hijo(nodo, "valor")
            .map(|v| self.expr(v))
            .unwrap_or_default();

        // Primera asignación simple a un nombre nuevo: en nuestro lenguaje eso
        // declara la variable. En Java usamos `var` para inferir su tipo.
        let ya_existe = self.locales.contains(&nombre) || self.globales.contains(&nombre);
        if operador == "=" && !ya_existe {
            self.locales.insert(nombre.clone());
            self.linea(&format!("var {} = {};", nombre, valor));
        } else {
            self.linea(&format!("{} {} {};", nombre, operador, valor));
        }
    }

    fn condicional(&mut self, nodo: &Nodo) {
        let cond = self.condicion_de(nodo);
        self.linea(&format!("if ({}) {{", cond));
        self.bloque(Self::hijo(nodo, "Cuerpo"));

        for rama in &nodo.children {
            match rama.value.as_str() {
                "elif" => {
                    let cond = self.condicion_de(rama);
                    self.linea(&format!("}} else if ({}) {{", cond));
                    self.bloque(Self::hijo(rama, "Cuerpo"));
                }
                "else" => {
                    self.linea("} else {");
                    self.bloque(Self::hijo(rama, "Cuerpo"));
                }
                _ => {}
            }
        }
        self.linea("}");
    }

    fn ciclo_while(&mut self, nodo: &Nodo) {
        let cond = self.condicion_de(nodo);
        self.linea(&format!("while ({}) {{", cond));
        self.bloque(Self::hijo(nodo, "Cuerpo"));
        self.linea("}");
    }

    fn ciclo_for(&mut self, nodo: &Nodo) {
        let variable = Self::valor_de(nodo, "variable").unwrap_or_default();
        self.locales.insert(variable.clone());
        let iterable = Self::hijo(nodo, "iterable").and_then(|i| i.children.first());

        match iterable {
            // for i in range(fin) -> for (int i = 0; i < fin; i++)
            Some(expr)
                if expr.value == "Llamada"
                    && Self::valor_de(expr, "nombre").as_deref() == Some("range") =>
            {
                let fin = Self::hijo(expr, "Args")
                    .and_then(|a| a.children.first())
                    .map(|arg| self.expr(arg))
                    .unwrap_or_else(|| "0".to_string());
                self.linea(&format!(
                    "for (int {v} = 0; {v} < {fin}; {v}++) {{",
                    v = variable,
                    fin = fin
                ));
            }
            // for c in cadena -> recorremos sus caracteres.
            Some(expr) => {
                let iter = self.expr(expr);
                self.linea(&format!("for (char {} : ({}).toCharArray()) {{", variable, iter));
            }
            None => self.linea(&format!("for (int {v} = 0; {v} < 0; {v}++) {{", v = variable)),
        }

        self.bloque(Self::hijo(nodo, "Cuerpo"));
        self.linea("}");
    }

    fn retorno(&mut self, nodo: &Nodo) {
        match Self::hijo(nodo, "valor") {
            Some(valor) => {
                let expr = self.expr(valor);
                self.linea(&format!("return {};", expr));
            }
            None => self.linea("return;"),
        }
    }

    /// Emite un bloque indentado a partir de su nodo "Cuerpo".
    fn bloque(&mut self, cuerpo: Option<&Nodo>) {
        self.nivel += 1;
        if let Some(c) = cuerpo {
            self.cuerpo(c);
        }
        self.nivel -= 1;
    }

    fn condicion_de(&mut self, nodo: &Nodo) -> String {
        Self::hijo(nodo, "Condición")
            .map(|c| self.expr(c))
            .unwrap_or_default()
    }

    // --- Expresiones -----------------------------------------------------

    fn expr(&self, nodo: &Nodo) -> String {
        // Una hoja (identificador o literal) no tiene hijos: se traduce directo.
        // Se resuelve primero para no confundir una variable llamada, por
        // ejemplo, `valor` o `iterable` con los nodos envoltorio del árbol.
        if nodo.children.is_empty() {
            return self.hoja(nodo.value.as_str());
        }

        let etiqueta = nodo.value.as_str();

        // Nodos que solo envuelven la expresión real.
        if matches!(etiqueta, "valor" | "Condición" | "iterable" | "Expresión") {
            return self.expr(&nodo.children[0]);
        }
        if etiqueta == "Llamada" {
            return self.llamada(nodo);
        }

        match etiqueta {
            "not" => format!("!({})", self.expr(&nodo.children[0])),
            "- (unario)" => format!("-({})", self.expr(&nodo.children[0])),
            "+ (unario)" => format!("+({})", self.expr(&nodo.children[0])),
            _ if nodo.children.len() == 2 => {
                let izq = self.expr(&nodo.children[0]);
                let der = self.expr(&nodo.children[1]);
                self.binaria(etiqueta, &izq, &der)
            }
            _ => String::new(),
        }
    }

    fn binaria(&self, op: &str, izq: &str, der: &str) -> String {
        match op {
            "and" => format!("({} && {})", izq, der),
            "or" => format!("({} || {})", izq, der),
            // Python `/` siempre es división real: forzamos punto flotante.
            "/" => format!("((double)({}) / ({}))", izq, der),
            "//" => format!("Math.floorDiv({}, {})", izq, der),
            "**" => format!("Math.pow({}, {})", izq, der),
            _ => format!("({} {} {})", izq, op, der),
        }
    }

    fn llamada(&self, nodo: &Nodo) -> String {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();
        let args: Vec<String> = Self::hijo(nodo, "Args")
            .map(|a| a.children.iter().map(|arg| self.expr(arg)).collect())
            .unwrap_or_default();

        // print(a, b, ...) une los argumentos con espacios, como en Python.
        if nombre == "print" {
            let contenido = if args.is_empty() {
                "\"\"".to_string()
            } else {
                args.join(" + \" \" + ")
            };
            return format!("System.out.println({})", contenido);
        }

        format!("{}({})", nombre, args.join(", "))
    }

    fn hoja(&self, lexema: &str) -> String {
        match lexema {
            "True" => return "true".to_string(),
            "False" => return "false".to_string(),
            "None" => return "null".to_string(),
            _ => {}
        }
        // Cadena con comillas simples -> comillas dobles de Java.
        if lexema.len() >= 2 && lexema.starts_with('\'') && lexema.ends_with('\'') {
            return format!("\"{}\"", &lexema[1..lexema.len() - 1]);
        }
        // Octal estilo Python (0o..) -> octal de Java (0..).
        if lexema.len() > 2 && lexema.to_lowercase().starts_with("0o") {
            return format!("0{}", &lexema[2..]);
        }
        lexema.to_string()
    }

    // --- Utilidades ------------------------------------------------------

    fn tipo_java(tipo: &str) -> String {
        match tipo {
            "int" => "int",
            "float" => "double",
            "str" => "String",
            "bool" => "boolean",
            "Void" => "void",
            otro => otro,
        }
        .to_string()
    }

    fn linea(&mut self, texto: &str) {
        self.salida.push_str(&"    ".repeat(self.nivel));
        self.salida.push_str(texto);
        self.salida.push('\n');
    }

    fn hijo<'a>(nodo: &'a Nodo, etiqueta: &str) -> Option<&'a Nodo> {
        nodo.children.iter().find(|h| h.value == etiqueta)
    }

    fn valor_de(nodo: &Nodo, prefijo: &str) -> Option<String> {
        let marca = format!("{}: ", prefijo);
        nodo.children
            .iter()
            .find_map(|h| h.value.strip_prefix(&marca).map(str::to_string))
    }
}
