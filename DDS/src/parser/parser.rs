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

use super::ast::{nodo, nodo_con, nodo_con_l, nodo_l, Nodo};

/// Operadores de asignación (simple y aumentada) reconocidos por el parser.
const OPS_ASIGNACION: &[&str] = &[
    "=", "+=", "-=", "*=", "/=", "%=", "//=", "**=", "&=", "|=", "^=", ">>=", "<<=",
];

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
    logger: Logger,
    hubo_error: bool,
}

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
        let linea = token.linea;
        let origen = format!("L{}:C{}", token.linea, token.columna);
        let cerca = if token.lexema.is_empty() {
            "<fin de archivo>".to_string()
        } else {
            format!("'{}'", token.lexema)
        };
        self.logger
            .error(format!("{} (cerca de {} en {})", mensaje.into(), cerca, origen));
        self.hubo_error = true;
        nodo_l("<error>", linea)
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
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let der = self.parse_y();
            izq = nodo_con_l(op, linea, vec![izq, der]);
        }
        izq
    }

    /// `and` lógico.
    fn parse_y(&mut self) -> Nodo {
        let mut izq = self.parse_no();
        while self.actual().lexema == "and" {
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let der = self.parse_no();
            izq = nodo_con_l(op, linea, vec![izq, der]);
        }
        izq
    }

    /// `not` lógico (unario prefijo).
    fn parse_no(&mut self) -> Nodo {
        if self.actual().lexema == "not" {
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let expr = self.parse_no();
            return nodo_con_l(op, linea, vec![expr]);
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
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let der = self.parse_suma();
            izq = nodo_con_l(op, linea, vec![izq, der]);
        }
        izq
    }

    /// Suma y resta.
    fn parse_suma(&mut self) -> Nodo {
        let mut izq = self.parse_termino();
        while matches!(self.actual().lexema.as_str(), "+" | "-") {
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let der = self.parse_termino();
            izq = nodo_con_l(op, linea, vec![izq, der]);
        }
        izq
    }

    /// Multiplicación, división, módulo y división entera.
    fn parse_termino(&mut self) -> Nodo {
        let mut izq = self.parse_unario();
        while matches!(self.actual().lexema.as_str(), "*" | "/" | "%" | "//") {
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let der = self.parse_unario();
            izq = nodo_con_l(op, linea, vec![izq, der]);
        }
        izq
    }

    /// Signo unario (`-x`, `+x`).
    fn parse_unario(&mut self) -> Nodo {
        if matches!(self.actual().lexema.as_str(), "-" | "+") {
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let expr = self.parse_unario();
            return nodo_con_l(format!("{} (unario)", op), linea, vec![expr]);
        }
        self.parse_potencia()
    }

    /// Potencia `**` (asociativa a la derecha, mayor precedencia binaria).
    fn parse_potencia(&mut self) -> Nodo {
        let base = self.parse_primario();
        if self.actual().lexema == "**" {
            let linea = self.actual().linea;
            let op = self.avanzar().lexema;
            let exponente = self.parse_unario();
            return nodo_con_l(op, linea, vec![base, exponente]);
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
                nodo_l(token.lexema, token.linea)
            }
            // Un identificador es una hoja, salvo que le siga '(' -> es llamada.
            TokenType::Identificador => {
                self.avanzar();
                if self.actual().lexema == "(" {
                    self.parse_llamada(token.lexema, token.linea)
                } else {
                    nodo_l(token.lexema, token.linea)
                }
            }
            // `None` (y otras palabras reservadas usadas como valor).
            TokenType::PalabraReservada if token.lexema == "None" => {
                self.avanzar();
                nodo_l(token.lexema, token.linea)
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
    fn parse_llamada(&mut self, nombre: String, linea: usize) -> Nodo {
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

        nodo_con_l(
            "Llamada",
            linea,
            vec![nodo(format!("nombre: {}", nombre)), args],
        )
    }

    // --- Sentencias que empiezan con un identificador --------------------
    //
    // Un identificador al inicio de una sentencia puede ser el comienzo de una
    // declaración (`x: int = ...`), una asignación (`x = ...`) o una expresión
    // suelta como una llamada (`print(...)`). Se decide con un token de mirada.

    fn parse_desde_identificador(&mut self) -> Nodo {
        let siguiente = self.mirar(1).lexema.clone();
        if siguiente == ":" {
            self.parse_declaracion()
        } else if OPS_ASIGNACION.contains(&siguiente.as_str()) {
            self.parse_asignacion()
        } else {
            // Sentencia de expresión (típicamente una llamada).
            let linea = self.actual().linea;
            let expr = self.parse_expresion();
            nodo_con_l("Expresión", linea, vec![expr])
        }
    }

    // --- Declaración: nombre ':' tipo [ '=' expresión ] ------------------

    fn parse_declaracion(&mut self) -> Nodo {
        let linea = self.actual().linea;
        let nombre = self.avanzar().lexema; // identificador declarado
        self.esperar_lexema(":");
        let tipo = self.avanzar().lexema; // tipo (int, float, str, bool, ...)

        let mut declaracion = nodo_l("Declaración", linea);
        declaracion.add_child_node(nodo(format!("tipo: {}", tipo)));
        declaracion.add_child_node(nodo(format!("nombre: {}", nombre)));

        // El valor inicial es opcional (p. ej. `y: int` sin asignar).
        if self.coincide_lexema("=") {
            let valor = self.parse_expresion();
            declaracion.add_child_node(nodo_con("valor", vec![valor]));
        }

        declaracion
    }

    // --- Asignación: nombre op_asig expresión ----------------------------

    fn parse_asignacion(&mut self) -> Nodo {
        let linea = self.actual().linea;
        let nombre = self.avanzar().lexema; // identificador destino
        let operador = self.avanzar().lexema; // '=' o forma aumentada

        let valor = self.parse_expresion();

        let mut asignacion = nodo_l("Asignación", linea);
        asignacion.add_child_node(nodo(format!("nombre: {}", nombre)));
        if operador != "=" {
            // Asignación aumentada (+=, -=, ...): dejamos constancia del operador.
            asignacion.add_child_node(nodo(format!("operador: {}", operador)));
        }
        asignacion.add_child_node(nodo_con("valor", vec![valor]));
        asignacion
    }

    // --- Despacho de sentencias ------------------------------------------

    /// Analiza una sentencia según con qué token empiece.
    fn parse_sentencia(&mut self) -> Nodo {
        let token = self.actual().clone();
        match token.tipo {
            TokenType::PalabraReservada => match token.lexema.as_str() {
                "def" => self.parse_funcion(),
                "if" => self.parse_condicional(),
                "while" => self.parse_while(),
                "for" => self.parse_for(),
                "return" => self.parse_return(),
                _ => {
                    let marcador =
                        self.error(format!("Sentencia no soportada: '{}'", token.lexema));
                    if !self.es_fin() {
                        self.avanzar();
                    }
                    marcador
                }
            },
            TokenType::Identificador => self.parse_desde_identificador(),
            _ => {
                let marcador = self.error("Se esperaba una sentencia");
                if !self.es_fin() {
                    self.avanzar();
                }
                marcador
            }
        }
    }

    /// ¿El token actual puede iniciar una expresión? (útil para `return`).
    fn inicia_expresion(&self) -> bool {
        let token = self.actual();
        match token.tipo {
            TokenType::Literal(_) | TokenType::Identificador => true,
            TokenType::PalabraReservada => matches!(token.lexema.as_str(), "None" | "not"),
            TokenType::Simbolo => matches!(token.lexema.as_str(), "(" | "-" | "+"),
            _ => false,
        }
    }

    // --- Bloques por indentación -----------------------------------------
    //
    // Python delimita los bloques por indentación. El lexer guarda en cada
    // token su `nivel` (sangría de la línea). Un bloque son las sentencias
    // cuyo nivel es mayor que el del encabezado (`if`, `while`, `def`, ...).

    fn parse_bloque(&mut self, nivel_padre: usize) -> Nodo {
        let mut cuerpo = nodo("Cuerpo");

        // El cuerpo debe estar más indentado que su encabezado.
        if self.es_fin() || self.actual().nivel <= nivel_padre {
            self.error("Se esperaba un bloque indentado");
            return cuerpo;
        }

        let nivel_bloque = self.actual().nivel;
        while !self.es_fin() && !self.hubo_error && self.actual().nivel >= nivel_bloque {
            let sentencia = self.parse_sentencia();
            cuerpo.add_child_node(sentencia);
        }
        cuerpo
    }

    // --- Condicional: if / elif / else -----------------------------------

    fn parse_condicional(&mut self) -> Nodo {
        let nivel_h = self.actual().nivel;
        let linea = self.actual().linea;
        self.avanzar(); // 'if'
        let condicion = self.parse_expresion();
        self.esperar_lexema(":");
        let cuerpo = self.parse_bloque(nivel_h);

        let mut nodo_if = nodo_l("if", linea);
        nodo_if.add_child_node(nodo_con("Condición", vec![condicion]));
        nodo_if.add_child_node(cuerpo);

        // Cadena de `elif`, siempre al mismo nivel de sangría que el `if`.
        while !self.es_fin()
            && !self.hubo_error
            && self.actual().lexema == "elif"
            && self.actual().nivel == nivel_h
        {
            let linea_elif = self.actual().linea;
            self.avanzar(); // 'elif'
            let cond = self.parse_expresion();
            self.esperar_lexema(":");
            let cu = self.parse_bloque(nivel_h);
            let mut nodo_elif = nodo_l("elif", linea_elif);
            nodo_elif.add_child_node(nodo_con("Condición", vec![cond]));
            nodo_elif.add_child_node(cu);
            nodo_if.add_child_node(nodo_elif);
        }

        // `else` opcional, también al mismo nivel.
        if !self.es_fin()
            && !self.hubo_error
            && self.actual().lexema == "else"
            && self.actual().nivel == nivel_h
        {
            self.avanzar(); // 'else'
            self.esperar_lexema(":");
            let cu = self.parse_bloque(nivel_h);
            nodo_if.add_child_node(nodo_con("else", vec![cu]));
        }

        nodo_if
    }

    // --- Bucle while -----------------------------------------------------

    fn parse_while(&mut self) -> Nodo {
        let nivel_h = self.actual().nivel;
        let linea = self.actual().linea;
        self.avanzar(); // 'while'
        let condicion = self.parse_expresion();
        self.esperar_lexema(":");
        let cuerpo = self.parse_bloque(nivel_h);

        nodo_con_l(
            "while",
            linea,
            vec![nodo_con("Condición", vec![condicion]), cuerpo],
        )
    }

    // --- Bucle for: for variable in iterable ------------------------------

    fn parse_for(&mut self) -> Nodo {
        let nivel_h = self.actual().nivel;
        let linea = self.actual().linea;
        self.avanzar(); // 'for'
        let variable = self.avanzar().lexema;
        self.esperar_lexema("in");
        let iterable = self.parse_expresion();
        self.esperar_lexema(":");
        let cuerpo = self.parse_bloque(nivel_h);

        nodo_con_l(
            "for",
            linea,
            vec![
                nodo(format!("variable: {}", variable)),
                nodo_con("iterable", vec![iterable]),
                cuerpo,
            ],
        )
    }

    // --- return [expresión] ----------------------------------------------

    fn parse_return(&mut self) -> Nodo {
        let linea = self.actual().linea;
        self.avanzar(); // 'return'
        let mut retorno = nodo_l("return", linea);

        // El valor es opcional; solo lo tomamos si va en la misma línea.
        if self.actual().linea == linea && self.inicia_expresion() {
            let valor = self.parse_expresion();
            retorno.add_child_node(nodo_con("valor", vec![valor]));
        }
        retorno
    }

    // --- Función: def nombre(parámetros) [-> tipo] : bloque --------------

    fn parse_funcion(&mut self) -> Nodo {
        let nivel_h = self.actual().nivel;
        let linea = self.actual().linea;
        self.avanzar(); // 'def'
        let nombre = self.avanzar().lexema; // nombre de la función

        self.esperar_lexema("(");
        let mut parametros = nodo("Parámetros");
        if self.actual().lexema != ")" {
            loop {
                let nombre_param = self.avanzar().lexema;
                // Tipo del parámetro opcional: `nombre: tipo`.
                if self.coincide_lexema(":") {
                    let tipo_param = self.avanzar().lexema;
                    parametros.add_child_node(nodo(format!("{}: {}", nombre_param, tipo_param)));
                } else {
                    parametros.add_child_node(nodo(nombre_param));
                }
                if !self.coincide_lexema(",") {
                    break;
                }
            }
        }
        self.esperar_lexema(")");

        // Tipo de retorno opcional (`-> tipo`); si no aparece, es Void.
        let retorno = if self.coincide_lexema("->") {
            self.avanzar().lexema
        } else {
            "Void".to_string()
        };
        self.esperar_lexema(":");
        let cuerpo = self.parse_bloque(nivel_h);

        nodo_con_l(
            "Función",
            linea,
            vec![
                nodo(format!("nombre: {}", nombre)),
                parametros,
                nodo(format!("retorno: {}", retorno)),
                cuerpo,
            ],
        )
    }

    // --- Punto de entrada ------------------------------------------------

    /// Analiza el programa completo. Devuelve `None` si hubo errores
    /// (el árbol queda cancelado, según la regla del proyecto).
    pub fn analizar(&mut self) -> Option<Tree<String>> {
        let mut raiz = nodo("root");

        // Un programa es una secuencia de sentencias al nivel superior.
        while !self.es_fin() && !self.hubo_error {
            let sentencia = self.parse_sentencia();
            raiz.add_child_node(sentencia);
        }

        // Regla del proyecto: ante cualquier error, se cancela el árbol.
        if self.hubo_error {
            return None;
        }

        let mut arbol = Tree::new();
        arbol.set_root_node(raiz);
        Some(arbol)
    }
}
