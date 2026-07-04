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

use super::ast::nodo;

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
