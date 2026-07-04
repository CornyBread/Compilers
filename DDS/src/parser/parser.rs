// Analizador Sintáctico (Parser) — Proyecto 3.
//
// Entrada : la lista de tokens producida por el analizador léxico.
// Salida  : un Árbol Sintáctico (AST) representado como `Tree<String>`.
//
// Regla del proyecto: si aparece un error sintáctico se cancela el árbol
// (se devuelve `None`) y se reporta el origen del error (línea y columna) a
// través del Logger.
//
// La técnica es descenso recursivo: cada regla de la gramática es un método.

use crate::lexer::token::{Token, TokenType};
use crate::logger::Logger;
use crate::tree::Tree;

use super::ast::{nodo, nodo_con, Nodo};

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
    logger: Logger,
    hubo_error: bool,
}

#[allow(dead_code)] // varios ayudantes se conectan en etapas posteriores
impl Parser {
    /// Construye el parser a partir de los tokens del lexer.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            i: 0,
            logger: Logger::named("parser"),
            hubo_error: false,
        }
    }

    /// Acceso de solo lectura al Logger (para reportar los errores al final).
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    // --- Navegación sobre el flujo de tokens -----------------------------

    /// Token actual (nunca se sale de rango: el último token es FinDeArchivo).
    fn actual(&self) -> &Token {
        let idx = self.i.min(self.tokens.len().saturating_sub(1));
        &self.tokens[idx]
    }

    /// Token que está `offset` posiciones más adelante (para mirar sin consumir).
    fn mirar(&self, offset: usize) -> &Token {
        let idx = (self.i + offset).min(self.tokens.len().saturating_sub(1));
        &self.tokens[idx]
    }

    /// Consume el token actual y devuelve una copia; no avanza más allá del EOF.
    fn avanzar(&mut self) -> Token {
        let token = self.actual().clone();
        if self.i < self.tokens.len() - 1 {
            self.i += 1;
        }
        token
    }

    /// ¿Ya llegamos al final del archivo?
    fn es_fin(&self) -> bool {
        matches!(self.actual().tipo, TokenType::FinDeArchivo)
    }

    /// Si el lexema actual coincide, lo consume y devuelve `true`.
    fn coincide_lexema(&mut self, lexema: &str) -> bool {
        if self.actual().lexema == lexema {
            self.avanzar();
            true
        } else {
            false
        }
    }

    /// Exige un lexema concreto; si no aparece, reporta un error sintáctico.
    fn esperar_lexema(&mut self, lexema: &str) {
        if !self.coincide_lexema(lexema) {
            self.error(format!("Se esperaba '{}'", lexema));
        }
    }

    // --- Reporte de errores ----------------------------------------------

    /// Registra un error sintáctico indicando el origen (línea y columna),
    /// marca el árbol como inválido y devuelve un nodo marcador `<error>`.
    fn error(&mut self, mensaje: impl Into<String>) -> super::ast::Nodo {
        let token = self.actual();
        let origen = format!("L{}:C{}", token.linea, token.columna);
        let cerca = if token.lexema.is_empty() {
            "<fin de archivo>".to_string()
        } else {
            format!("'{}'", token.lexema)
        };
        self.logger
            .error(format!("{} (cerca de {} en {})", mensaje.into(), cerca, origen));
        self.hubo_error = true;
        nodo("<error>")
    }

    // --- Operación: expresiones con precedencia --------------------------
    //
    // Se resuelve con descenso recursivo por niveles de precedencia (de menor
    // a mayor): or -> and -> not -> comparación -> +,- -> *,/,%,// -> ** ->
    // unario -> primario. Cada operador binario crea un nodo cuya etiqueta es
    // el operador y cuyos dos hijos son los operandos, como en el pizarrón.

    /// Punto de entrada de una expresión.
    fn parse_expresion(&mut self) -> Nodo {
        self.parse_o()
    }

    /// `or` lógico (menor precedencia).
    fn parse_o(&mut self) -> Nodo {
        let mut izq = self.parse_y();
        while self.actual().lexema == "or" {
            let op = self.avanzar().lexema;
            let der = self.parse_y();
            izq = nodo_con(op, vec![izq, der]);
        }
        izq
    }

    /// `and` lógico.
    fn parse_y(&mut self) -> Nodo {
        let mut izq = self.parse_no();
        while self.actual().lexema == "and" {
            let op = self.avanzar().lexema;
            let der = self.parse_no();
            izq = nodo_con(op, vec![izq, der]);
        }
        izq
    }

    /// `not` lógico (unario prefijo).
    fn parse_no(&mut self) -> Nodo {
        if self.actual().lexema == "not" {
            let op = self.avanzar().lexema;
            let expr = self.parse_no();
            return nodo_con(op, vec![expr]);
        }
        self.parse_comparacion()
    }

    /// Comparaciones: <, >, <=, >=, ==, !=
    fn parse_comparacion(&mut self) -> Nodo {
        let mut izq = self.parse_suma();
        while matches!(
            self.actual().lexema.as_str(),
            "<" | ">" | "<=" | ">=" | "==" | "!="
        ) {
            let op = self.avanzar().lexema;
            let der = self.parse_suma();
            izq = nodo_con(op, vec![izq, der]);
        }
        izq
    }

    /// Suma y resta.
    fn parse_suma(&mut self) -> Nodo {
        let mut izq = self.parse_termino();
        while matches!(self.actual().lexema.as_str(), "+" | "-") {
            let op = self.avanzar().lexema;
            let der = self.parse_termino();
            izq = nodo_con(op, vec![izq, der]);
        }
        izq
    }

    /// Multiplicación, división, módulo y división entera.
    fn parse_termino(&mut self) -> Nodo {
        let mut izq = self.parse_unario();
        while matches!(self.actual().lexema.as_str(), "*" | "/" | "%" | "//") {
            let op = self.avanzar().lexema;
            let der = self.parse_unario();
            izq = nodo_con(op, vec![izq, der]);
        }
        izq
    }

    /// Signo unario (`-x`, `+x`).
    fn parse_unario(&mut self) -> Nodo {
        if matches!(self.actual().lexema.as_str(), "-" | "+") {
            let op = self.avanzar().lexema;
            let expr = self.parse_unario();
            return nodo_con(format!("{} (unario)", op), vec![expr]);
        }
        self.parse_potencia()
    }

    /// Potencia `**` (asociativa a la derecha, mayor precedencia binaria).
    fn parse_potencia(&mut self) -> Nodo {
        let base = self.parse_primario();
        if self.actual().lexema == "**" {
            let op = self.avanzar().lexema;
            let exponente = self.parse_unario();
            return nodo_con(op, vec![base, exponente]);
        }
        base
    }

    /// Elemento primario: literal, identificador, llamada o `( expresión )`.
    fn parse_primario(&mut self) -> Nodo {
        let token = self.actual().clone();
        match token.tipo {
            // Un valor literal (entero, flotante, cadena, booleano) es una hoja.
            TokenType::Literal(_) => {
                self.avanzar();
                nodo(token.lexema)
            }
            // Un identificador es una hoja, salvo que le siga '(' -> es llamada.
            TokenType::Identificador => {
                self.avanzar();
                if self.actual().lexema == "(" {
                    self.parse_llamada(token.lexema)
                } else {
                    nodo(token.lexema)
                }
            }
            // `None` (y otras palabras reservadas usadas como valor).
            TokenType::PalabraReservada if token.lexema == "None" => {
                self.avanzar();
                nodo(token.lexema)
            }
            // Expresión entre paréntesis.
            TokenType::Simbolo if token.lexema == "(" => {
                self.avanzar();
                let expr = self.parse_expresion();
                self.esperar_lexema(")");
                expr
            }
            _ => {
                let marcador = self.error("Se esperaba una expresión");
                // Consumimos un token para garantizar avance y evitar bucles.
                if !self.es_fin() {
                    self.avanzar();
                }
                marcador
            }
        }
    }

    // --- Llamada: identificador '(' argumentos ')' -----------------------

    /// Analiza una llamada a función `nombre(arg1, arg2, ...)`.
    /// Al entrar, el token actual es el '(' de apertura.
    fn parse_llamada(&mut self, nombre: String) -> Nodo {
        self.avanzar(); // consume '('
        let mut args = nodo("Args");
        if self.actual().lexema != ")" {
            loop {
                let arg = self.parse_expresion();
                args.add_child_node(arg);
                if !self.coincide_lexema(",") {
                    break;
                }
            }
        }
        self.esperar_lexema(")");

        nodo_con(
            "Llamada",
            vec![nodo(format!("nombre: {}", nombre)), args],
        )
    }

    // --- Punto de entrada ------------------------------------------------

    /// Analiza el programa completo. Devuelve `None` si hubo errores
    /// (el árbol queda cancelado, según la regla del proyecto).
    pub fn analizar(&mut self) -> Option<Tree<String>> {
        // En esta primera etapa solo montamos la raíz del árbol; las reglas
        // (declaración, asignación, operación, etc.) se agregan más adelante.
        let raiz = nodo("root");

        if self.hubo_error {
            return None;
        }

        let mut arbol = Tree::new();
        arbol.set_root_node(raiz);
        Some(arbol)
    }
}
