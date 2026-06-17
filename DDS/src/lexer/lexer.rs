use crate::file_reader::FileReader;
use crate::logger::Logger;
use super::token::{Literal, Token, TokenType};

const KEYWORDS: &[&str] = &[
    "None", "and", "as", "assert", "async", "await", "break", "class",
    "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
    "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
    "try", "while", "with", "yield",
];

const OP3: &[&str] = &["**=", "//=", ">>=", "<<=", "..."];
const OP2: &[&str] = &[
    "==", "!=", "<=", ">=", "**", "//", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "->",
    ":=", "<<", ">>",
];
const OP1: &[&str] = &[
    "+", "-", "*", "/", "%", "=", "<", ">", "(", ")", "[", "]", "{", "}", ":", ",", ".", ";",
    "@", "&", "|", "^", "~",
];

pub struct Lexer {
    chars: Vec<char>,
    i: usize,        
    linea: usize,    
    col: usize,      
    nivel: usize,    
    logger: Logger,
}

impl Lexer {
    #[allow(dead_code)]
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            i: 0,
            linea: 1,
            col: 1,
            nivel: 0,
            logger: Logger::named("lexer"),
        }
    }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let contenido = FileReader::read_all(path)?;
        Ok(Self::new(&contenido))
    }

    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut inicio_linea = true;

        loop {
            if inicio_linea {
                self.medir_indentacion();
                inicio_linea = false;
            }

            match self.peek() {
                None => break,
                Some('\n') => {
                    self.avanzar();
                    inicio_linea = true;
                }
                Some(c) if c == ' ' || c == '\t' || c == '\r' => {
                    self.avanzar();
                }
                Some('#') => {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' {
                            break;
                        }
                        self.avanzar();
                    }
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let t = self.leer_identificador();
                    tokens.push(t);
                }
                Some(c) if c.is_ascii_digit() => {
                    let t = self.leer_numero();
                    tokens.push(t);
                }
                Some(c) if c == '"' || c == '\'' => {
                    let t = self.leer_cadena();
                    tokens.push(t);
                }
                Some(_) => {
                    if let Some(t) = self.leer_simbolo() {
                        tokens.push(t);
                    } else {
                        let (l, c, n) = (self.linea, self.col, self.nivel);
                        let ch = self.avanzar().unwrap();
                        self.logger
                            .warn(format!("Carácter desconocido '{}' en L{}:C{}", ch, l, c));
                        tokens.push(Token::new(TokenType::Desconocido, ch.to_string(), l, c, n));
                    }
                }
            }
        }

        tokens.push(Token::new(
            TokenType::FinDeArchivo,
            "",
            self.linea,
            self.col,
            self.nivel,
        ));
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek_en(&self, offset: usize) -> Option<char> {
        self.chars.get(self.i + offset).copied()
    }

    fn avanzar(&mut self) -> Option<char> {
        let c = self.chars.get(self.i).copied();
        if let Some(ch) = c {
            self.i += 1;
            if ch == '\n' {
                self.linea += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    fn slice(&self, n: usize) -> String {
        self.chars[self.i..].iter().take(n).collect()
    }

    fn medir_indentacion(&mut self) {
        let mut nivel = 0;
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' {
                nivel += 1;
                self.avanzar();
            } else {
                break;
            }
        }
        self.nivel = nivel;
    }

    fn leer_identificador(&mut self) -> Token {
        let (l, c, n) = (self.linea, self.col, self.nivel);
        let mut lex = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                lex.push(ch);
                self.avanzar();
            } else {
                break;
            }
        }

        let tipo = if lex == "True" || lex == "False" {
            TokenType::Literal(Literal::Booleano)
        } else if KEYWORDS.contains(&lex.as_str()) {
            TokenType::PalabraReservada
        } else {
            TokenType::Identificador
        };

        Token::new(tipo, lex, l, c, n)
    }

    fn leer_numero(&mut self) -> Token {
        let (l, c, n) = (self.linea, self.col, self.nivel);
        let mut lex = String::new();
        let mut es_flotante = false;

        let consumir_digitos = |lexer: &mut Self, lex: &mut String| {
            while let Some(ch) = lexer.peek() {
                if ch.is_ascii_digit() || ch == '_' {
                    lex.push(ch);
                    lexer.avanzar();
                } else {
                    break;
                }
            }
        };

        // Prefijos no decimales (siempre enteros): 0x.. hex, 0o.. octal, 0b.. binario.
        if self.peek() == Some('0') {
            let base = match self.peek_en(1) {
                Some('x') | Some('X') => Some(16u32),
                Some('o') | Some('O') => Some(8),
                Some('b') | Some('B') => Some(2),
                _ => None,
            };
            if let Some(base) = base {
                lex.push(self.avanzar().unwrap()); // '0'
                lex.push(self.avanzar().unwrap()); // x / o / b
                while let Some(ch) = self.peek() {
                    if ch == '_' || ch.is_digit(base) {
                        lex.push(ch);
                        self.avanzar();
                    } else {
                        break;
                    }
                }
                return Token::new(TokenType::Literal(Literal::Entero), lex, l, c, n);
            }
        }

        consumir_digitos(self, &mut lex);

        // Parte decimal: soporta "1.5" y también "1." (punto final), pero no "1.." (rango).
        if self.peek() == Some('.') && self.peek_en(1) != Some('.') {
            es_flotante = true;
            lex.push('.');
            self.avanzar();
            consumir_digitos(self, &mut lex);
        }

        if matches!(self.peek(), Some('e') | Some('E')) {
            let exponente_valido = match self.peek_en(1) {
                Some('+') | Some('-') => self.peek_en(2).is_some_and(|c| c.is_ascii_digit()),
                Some(c) => c.is_ascii_digit(),
                None => false,
            };
            if exponente_valido {
                es_flotante = true;
                lex.push(self.avanzar().unwrap()); // e / E
                if matches!(self.peek(), Some('+') | Some('-')) {
                    lex.push(self.avanzar().unwrap());
                }
                consumir_digitos(self, &mut lex);
            }
        }

        let tipo = if es_flotante { Literal::Flotante } else { Literal::Entero };
        Token::new(TokenType::Literal(tipo), lex, l, c, n)
    }

    fn leer_cadena(&mut self) -> Token {
        let (l, c, n) = (self.linea, self.col, self.nivel);
        let comilla = self.peek().unwrap();
        let triple = self.peek_en(1) == Some(comilla) && self.peek_en(2) == Some(comilla);
        let mut lex = String::new();

        if triple {
            for _ in 0..3 {
                lex.push(self.avanzar().unwrap());
            }
            loop {
                match self.peek() {
                    None => {
                        self.logger
                            .warn(format!("Cadena triple sin cerrar en L{}:C{}", l, c));
                        return Token::new(TokenType::Desconocido, lex, l, c, n);
                    }
                    Some(ch) => {
                        if ch == comilla
                            && self.peek_en(1) == Some(comilla)
                            && self.peek_en(2) == Some(comilla)
                        {
                            for _ in 0..3 {
                                lex.push(self.avanzar().unwrap());
                            }
                            break;
                        }
                        lex.push(ch);
                        self.avanzar();
                    }
                }
            }
        } else {
            lex.push(self.avanzar().unwrap()); // comilla de apertura
            loop {
                match self.peek() {
                    None | Some('\n') => {
                        self.logger
                            .warn(format!("Cadena sin cerrar en L{}:C{}", l, c));
                        return Token::new(TokenType::Desconocido, lex, l, c, n);
                    }
                    Some('\\') => {
                        lex.push(self.avanzar().unwrap());
                        if self.peek().is_some() {
                            lex.push(self.avanzar().unwrap());
                        }
                    }
                    Some(ch) if ch == comilla => {
                        lex.push(self.avanzar().unwrap());
                        break;
                    }
                    Some(ch) => {
                        lex.push(ch);
                        self.avanzar();
                    }
                }
            }
        }

        Token::new(TokenType::Literal(Literal::Cadena), lex, l, c, n)
    }

    fn leer_simbolo(&mut self) -> Option<Token> {
        let (l, c, n) = (self.linea, self.col, self.nivel);
        let resto3 = self.slice(3);
        let resto2 = self.slice(2);
        let resto1 = self.slice(1);

        let elegido = if OP3.contains(&resto3.as_str()) {
            Some((3, resto3))
        } else if OP2.contains(&resto2.as_str()) {
            Some((2, resto2))
        } else if OP1.contains(&resto1.as_str()) {
            Some((1, resto1))
        } else {
            None
        };

        elegido.map(|(cantidad, lex)| {
            for _ in 0..cantidad {
                self.avanzar();
            }
            Token::new(TokenType::Simbolo, lex, l, c, n)
        })
    }
}


