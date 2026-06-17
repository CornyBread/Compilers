use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Entero,
    Flotante,
    Cadena,
    Booleano,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let etiqueta = match self {
            Literal::Entero => "Entero",
            Literal::Flotante => "Flotante",
            Literal::Cadena => "Cadena",
            Literal::Booleano => "Booleano",
        };
        write!(f, "{}", etiqueta)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    PalabraReservada,
    Identificador,
    Simbolo,
    Literal(Literal), // categoría "Valor"
    FinDeArchivo,
    Desconocido,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::PalabraReservada => write!(f, "PalabraReservada"),
            TokenType::Identificador => write!(f, "Identificador"),
            TokenType::Simbolo => write!(f, "Simbolo"),
            TokenType::Literal(tipo) => write!(f, "Valor({})", tipo),
            TokenType::FinDeArchivo => write!(f, "FinDeArchivo"),
            TokenType::Desconocido => write!(f, "Desconocido"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub tipo: TokenType,
    pub lexema: String,
    pub linea: usize,
    pub columna: usize,
    pub nivel: usize,
}

impl Token {
    pub fn new(tipo: TokenType, lexema: impl Into<String>, linea: usize, columna: usize, nivel: usize) -> Self {
        Token {
            tipo,
            lexema: lexema.into(),
            linea,
            columna,
            nivel,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[L{}:C{} nivel {}] {:<18} '{}'",
            self.linea, self.columna, self.nivel, self.tipo, self.lexema
        )
    }
}
