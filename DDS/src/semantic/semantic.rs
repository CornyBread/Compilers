// Analizador Semántico — Proyecto 4.
//
// Entrada : el árbol sintáctico generado por el parser.
// Salida  : la Tabla de Símbolos con todas las declaraciones del programa
//           (variables, parámetros y funciones) + errores y advertencias.
//
// El árbol se recorre UNA sola vez, de izquierda a derecha. Cada declaración
// queda en la tabla con su ámbito, de modo que en la fase de ejecución baste
// consultar la tabla sin volver a recorrer el árbol.
//
// Ámbitos: cada función abre su propio ámbito (p. ej. "main") y cada bloque
// if/elif/else/while/for abre un sub-ámbito ("main::if#1"). Un nombre se
// resuelve buscando del ámbito activo más interno hacia afuera; si su ámbito
// ya se cerró, el nombre no se encuentra.
//
// Casos verificados (pizarrón 21/7):
//   1. Redeclaraciones           -> error
//   2. Tipos inválidos y arrays  -> error
//   3. Datos fuera de bloque     -> error
//   4. Funciones dentro de funciones -> error
//   5. Función main()            -> error si no existe
//   6. Código fuera de funciones -> error (excepto declaraciones y main())
//   7. Argumentos (cantidad y tipo en llamadas) -> error
//   8. Dead code                 -> advertencia (warning)

use crate::logger::Logger;
use crate::parser::ast::Nodo;
use crate::tree::Tree;

use super::tabla::{Categoria, Simbolo, TablaSimbolos};

/// Tipos aceptados en las anotaciones del lenguaje.
const TIPOS_VALIDOS: &[&str] = &["int", "float", "str", "bool"];
/// Colecciones de Python que este proyecto NO soporta (caso "Arrays").
const TIPOS_ARRAY: &[&str] = &["list", "tuple", "dict", "set"];
/// Marcador interno: la expresión ya falló; evita errores en cascada.
const TIPO_ERROR: &str = "<error>";

pub struct AnalizadorSemantico {
    tabla: TablaSimbolos,
    logger: Logger,
    /// Pila de ámbitos activos; el último es el más interno.
    ambitos: Vec<String>,
    /// Contador para dar nombre único a cada bloque (if#1, while#2, ...).
    bloques: usize,
    /// Función que se está analizando (None = nivel superior del programa).
    funcion_actual: Option<String>,
    /// Tipo de retorno declarado por la función actual.
    retorno_actual: String,
    /// ¿La función actual ya tuvo al menos un return?
    tiene_return: bool,
    hubo_error: bool,
}

impl AnalizadorSemantico {
    pub fn new() -> Self {
        let mut tabla = TablaSimbolos::new();

        // Funciones nativas del lenguaje: se registran de una vez en la tabla
        // para que las llamadas a `print` y `range` resuelvan como cualquier
        // otra función. Se marcan como usadas para no avisar dead code.
        let mut print_fn = Simbolo::funcion("print", "Void", "global", Vec::new());
        print_fn.variadica = true;
        print_fn.usado = true;
        tabla.declarar(print_fn);

        let mut range_fn = Simbolo::funcion(
            "range",
            "range",
            "global",
            vec![("fin".to_string(), "int".to_string())],
        );
        range_fn.usado = true;
        tabla.declarar(range_fn);

        AnalizadorSemantico {
            tabla,
            logger: Logger::named("semantico"),
            ambitos: vec!["global".to_string()],
            bloques: 0,
            funcion_actual: None,
            retorno_actual: "Void".to_string(),
            tiene_return: false,
            hubo_error: false,
        }
    }

    pub fn tabla(&self) -> &TablaSimbolos {
        &self.tabla
    }

    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    #[allow(dead_code)]
    pub fn hubo_error(&self) -> bool {
        self.hubo_error
    }

    // --- Reporte -----------------------------------------------------------

    fn error(&mut self, mensaje: impl Into<String>) {
        self.logger.error(mensaje.into());
        self.hubo_error = true;
    }

    fn advertencia(&mut self, mensaje: impl Into<String>) {
        self.logger.warn(mensaje.into());
    }

    // --- Utilidades sobre el árbol de etiquetas ----------------------------

    /// Busca el hijo con la etiqueta exacta (p. ej. "Cuerpo", "Args").
    fn hijo<'a>(nodo: &'a Nodo, etiqueta: &str) -> Option<&'a Nodo> {
        nodo.children.iter().find(|h| h.value == etiqueta)
    }

    /// Extrae "X" de un hijo con etiqueta "prefijo: X" (p. ej. "nombre: main").
    fn valor_de(nodo: &Nodo, prefijo: &str) -> Option<String> {
        let marca = format!("{}: ", prefijo);
        nodo.children
            .iter()
            .find_map(|h| h.value.strip_prefix(&marca).map(str::to_string))
    }

    /// Ámbito activo más interno (donde se declara lo nuevo).
    fn ambito_actual(&self) -> String {
        self.ambitos.last().cloned().unwrap_or_else(|| "global".to_string())
    }

    /// Abre un sub-ámbito con nombre único, p. ej. "main::if#1".
    fn abrir_bloque(&mut self, clase: &str) -> String {
        self.bloques += 1;
        let nombre = format!("{}::{}#{}", self.ambito_actual(), clase, self.bloques);
        self.ambitos.push(nombre.clone());
        nombre
    }

    fn cerrar_bloque(&mut self) {
        self.ambitos.pop();
    }

    // --- Punto de entrada --------------------------------------------------

    /// Recorre el programa completo (una sola pasada, izquierda a derecha).
    pub fn analizar(&mut self, arbol: &Tree<String>) {
        let raiz = match arbol.root() {
            Some(r) => r,
            None => return,
        };

        for sentencia in &raiz.children {
            self.analizar_nivel_superior(sentencia);
        }

        // Caso 5: el programa debe definir la función main().
        let hay_main = self
            .tabla
            .simbolos()
            .iter()
            .any(|s| s.nombre == "main" && s.categoria == Categoria::Funcion);
        if !hay_main {
            self.error("Función main: el programa no define la función main()");
        }

        // Caso 8 (dead code): símbolos declarados que nunca se usan.
        let sin_usar: Vec<String> = self
            .tabla
            .simbolos()
            .iter()
            .filter(|s| !s.usado && s.nombre != "main")
            .filter(|s| s.categoria != Categoria::Parametro)
            .map(|s| match s.categoria {
                Categoria::Funcion => {
                    format!("Dead code: la función '{}' nunca se llama", s.nombre)
                }
                _ => format!(
                    "Dead code: la variable '{}' (ámbito '{}') nunca se usa",
                    s.nombre, s.ambito
                ),
            })
            .collect();
        for mensaje in sin_usar {
            self.advertencia(mensaje);
        }
    }

    // --- Nivel superior ----------------------------------------------------
    //
    // Caso 6 del pizarrón: fuera de las funciones solo se permiten
    // declaraciones (de variables y de funciones) y la llamada a main().

    fn analizar_nivel_superior(&mut self, sentencia: &Nodo) {
        match sentencia.value.as_str() {
            "Función" => self.analizar_funcion(sentencia),
            "Declaración" => self.analizar_declaracion(sentencia),
            "Expresión" => {
                // La única expresión permitida al nivel superior es la
                // llamada que arranca el programa: main().
                let llamada = sentencia.children.first();
                let es_main = llamada.map_or(false, |n| {
                    n.value == "Llamada" && Self::valor_de(n, "nombre").as_deref() == Some("main")
                });
                if es_main {
                    self.tipo_expresion(sentencia);
                } else {
                    self.error(
                        "Código fuera de funciones: al nivel superior solo se permiten \
                         declaraciones, definiciones de función y la llamada a main()",
                    );
                }
            }
            "return" => self.error("Código fuera de funciones: 'return' fuera de una función"),
            "<error>" => {}
            otra => self.error(format!(
                "Código fuera de funciones: la sentencia '{}' debe ir dentro de una función \
                 (solo se permiten declaraciones)",
                otra
            )),
        }
    }

    // --- Funciones ---------------------------------------------------------

    fn analizar_funcion(&mut self, nodo: &Nodo) {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();

        // Caso 4: no se permiten funciones dentro de funciones.
        if let Some(externa) = &self.funcion_actual {
            self.error(format!(
                "Funciones anidadas: la función '{}' no puede definirse dentro de '{}'",
                nombre, externa
            ));
            return; // no se registra ni se analiza su cuerpo
        }

        let retorno = Self::valor_de(nodo, "retorno").unwrap_or_else(|| "Void".to_string());
        if retorno != "Void" {
            self.validar_tipo(&retorno, &format!("el retorno de la función '{}'", nombre));
        }

        // Parámetros: "n: int" (con tipo) o "n" (sin tipo -> error).
        let mut parametros: Vec<(String, String)> = Vec::new();
        if let Some(nodo_params) = Self::hijo(nodo, "Parámetros") {
            for param in &nodo_params.children {
                match param.value.split_once(": ") {
                    Some((p_nombre, p_tipo)) => {
                        self.validar_tipo(
                            p_tipo,
                            &format!("el parámetro '{}' de '{}'", p_nombre, nombre),
                        );
                        parametros.push((p_nombre.to_string(), p_tipo.to_string()));
                    }
                    None => {
                        self.error(format!(
                            "Tipos inválidos: el parámetro '{}' de la función '{}' debe \
                             declarar su tipo (nombre: tipo)",
                            param.value, nombre
                        ));
                        parametros.push((param.value.clone(), TIPO_ERROR.to_string()));
                    }
                }
            }
        }

        // Caso 5: main() no recibe parámetros.
        if nombre == "main" && !parametros.is_empty() {
            self.error("Función main: main() no debe recibir parámetros");
        }

        // Caso 1: redeclaración de función. Si ya existía, no se analiza el
        // cuerpo duplicado (evita arrastrar errores de la primera versión).
        let simbolo = Simbolo::funcion(nombre.clone(), retorno.clone(), "global", parametros.clone());
        if !self.tabla.declarar(simbolo) {
            self.error(format!(
                "Redeclaración: la función '{}' ya estaba declarada",
                nombre
            ));
            return;
        }

        // Se abre el ámbito de la función y se registran sus parámetros.
        self.ambitos.push(nombre.clone());
        self.funcion_actual = Some(nombre.clone());
        self.retorno_actual = retorno.clone();
        self.tiene_return = false;

        for (p_nombre, p_tipo) in &parametros {
            let parametro = Simbolo::parametro(p_nombre.clone(), p_tipo.clone(), nombre.clone());
            if !self.tabla.declarar(parametro) {
                self.error(format!(
                    "Redeclaración: el parámetro '{}' está repetido en la función '{}'",
                    p_nombre, nombre
                ));
            }
        }

        if let Some(cuerpo) = Self::hijo(nodo, "Cuerpo") {
            self.analizar_cuerpo(cuerpo);
        }

        // Una función con retorno declarado debería tener al menos un return.
        if retorno != "Void" && !self.tiene_return {
            self.advertencia(format!(
                "La función '{}' declara retorno '{}' pero no tiene ningún return",
                nombre, retorno
            ));
        }

        self.funcion_actual = None;
        self.retorno_actual = "Void".to_string();
        self.ambitos.pop();
    }

    // --- Cuerpos y sentencias ----------------------------------------------

    /// Analiza las sentencias de un bloque, en orden (izquierda a derecha).
    /// Caso 8: lo que siga a un `return` dentro del mismo bloque es dead code.
    fn analizar_cuerpo(&mut self, cuerpo: &Nodo) {
        let mut return_visto = false;
        let mut muertas = 0;

        for sentencia in &cuerpo.children {
            if return_visto {
                muertas += 1;
            }
            if sentencia.value == "return" {
                return_visto = true;
            }
            self.analizar_sentencia(sentencia);
        }

        if muertas > 0 {
            self.advertencia(format!(
                "Dead code: {} sentencia(s) inalcanzable(s) después del 'return' en '{}'",
                muertas,
                self.ambito_actual()
            ));
        }
    }

    fn analizar_sentencia(&mut self, sentencia: &Nodo) {
        match sentencia.value.as_str() {
            "Declaración" => self.analizar_declaracion(sentencia),
            "Asignación" => self.analizar_asignacion(sentencia),
            "Expresión" => {
                self.tipo_expresion(sentencia);
            }
            "if" => self.analizar_if(sentencia),
            "while" => self.analizar_while(sentencia),
            "for" => self.analizar_for(sentencia),
            "return" => self.analizar_return(sentencia),
            "Función" => self.analizar_funcion(sentencia), // detecta la anidación
            _ => {}
        }
    }

    // --- Declaración y asignación ------------------------------------------

    fn analizar_declaracion(&mut self, nodo: &Nodo) {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();
        let tipo = Self::valor_de(nodo, "tipo").unwrap_or_default();

        // Caso 2: el tipo anotado debe existir en el lenguaje.
        let tipo_ok = self.validar_tipo(&tipo, &format!("la variable '{}'", nombre));
        let tipo_declarado = if tipo_ok { tipo.clone() } else { TIPO_ERROR.to_string() };

        // Valor inicial opcional: su tipo debe ser compatible con el anotado.
        let mut inicializado = false;
        if let Some(valor) = Self::hijo(nodo, "valor") {
            inicializado = true;
            let tipo_valor = self.tipo_expresion(valor);
            if !Self::compatibles(&tipo_declarado, &tipo_valor) {
                self.error(format!(
                    "Tipos inválidos: no se puede asignar un valor '{}' a la variable \
                     '{}' declarada como '{}'",
                    tipo_valor, nombre, tipo
                ));
            }
        }

        // Caso 1: redeclaración en el mismo ámbito.
        let simbolo = Simbolo::variable(
            nombre.clone(),
            tipo_declarado,
            self.ambito_actual(),
            inicializado,
        );
        if !self.tabla.declarar(simbolo) {
            self.error(format!(
                "Redeclaración: '{}' ya existe en el ámbito '{}'",
                nombre,
                self.ambito_actual()
            ));
        }
    }

    fn analizar_asignacion(&mut self, nodo: &Nodo) {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();
        let operador = Self::valor_de(nodo, "operador"); // solo en aumentadas (+=, -=, ...)

        let tipo_valor = match Self::hijo(nodo, "valor") {
            Some(valor) => self.tipo_expresion(valor),
            None => TIPO_ERROR.to_string(),
        };

        // ¿El destino ya existe en algún ámbito activo?
        let destino = self
            .tabla
            .resolver_mut(&nombre, &self.ambitos)
            .map(|s| (s.categoria.clone(), s.tipo.clone()));

        match destino {
            Some((Categoria::Funcion, _)) => {
                self.error(format!(
                    "Tipos inválidos: no se puede asignar un valor a la función '{}'",
                    nombre
                ));
            }
            Some((_, tipo_destino)) => {
                // En una asignación aumentada (x += v) primero se opera
                // x (op) v y ese resultado es el que se guarda en x.
                let tipo_final = match &operador {
                    Some(op) => {
                        let base = op.trim_end_matches('=');
                        self.tipo_binario(base, &tipo_destino, &tipo_valor)
                    }
                    None => tipo_valor,
                };
                if !Self::compatibles(&tipo_destino, &tipo_final) {
                    self.error(format!(
                        "Tipos inválidos: no se puede asignar un valor '{}' a '{}' \
                         que es de tipo '{}'",
                        tipo_final, nombre, tipo_destino
                    ));
                }
                if let Some(simbolo) = self.tabla.resolver_mut(&nombre, &self.ambitos) {
                    simbolo.inicializado = true;
                    if operador.is_some() {
                        simbolo.usado = true; // x += v también LEE x
                    }
                }
            }
            None => {
                if operador.is_some() {
                    // Caso 3: x += v exige que x ya exista.
                    self.error(format!(
                        "Datos fuera de bloque: la variable '{}' no está declarada en \
                         ningún ámbito activo",
                        nombre
                    ));
                } else if tipo_valor == "Void" {
                    self.error(format!(
                        "Tipos inválidos: no se puede asignar a '{}' el resultado de una \
                         función que no retorna valor",
                        nombre
                    ));
                } else {
                    // Primera asignación = declaración implícita (estilo Python):
                    // la variable nace en el ámbito actual con el tipo inferido.
                    let simbolo = Simbolo::variable(
                        nombre.clone(),
                        tipo_valor,
                        self.ambito_actual(),
                        true,
                    );
                    self.tabla.declarar(simbolo);
                }
            }
        }
    }

    // --- Estructuras de control --------------------------------------------

    fn analizar_condicion(&mut self, nodo: &Nodo, construccion: &str) {
        if let Some(condicion) = Self::hijo(nodo, "Condición") {
            let tipo = self.tipo_expresion(condicion);
            if tipo != "bool" && tipo != TIPO_ERROR {
                self.error(format!(
                    "Tipos inválidos: la condición del '{}' debe ser bool, se obtuvo '{}'",
                    construccion, tipo
                ));
            }
        }
    }

    fn analizar_if(&mut self, nodo: &Nodo) {
        self.analizar_condicion(nodo, "if");
        if let Some(cuerpo) = Self::hijo(nodo, "Cuerpo") {
            self.abrir_bloque("if");
            self.analizar_cuerpo(cuerpo);
            self.cerrar_bloque();
        }

        // Las ramas elif/else cuelgan del mismo nodo `if`.
        for rama in &nodo.children {
            match rama.value.as_str() {
                "elif" => {
                    self.analizar_condicion(rama, "elif");
                    if let Some(cuerpo) = Self::hijo(rama, "Cuerpo") {
                        self.abrir_bloque("elif");
                        self.analizar_cuerpo(cuerpo);
                        self.cerrar_bloque();
                    }
                }
                "else" => {
                    if let Some(cuerpo) = Self::hijo(rama, "Cuerpo") {
                        self.abrir_bloque("else");
                        self.analizar_cuerpo(cuerpo);
                        self.cerrar_bloque();
                    }
                }
                _ => {}
            }
        }
    }

    fn analizar_while(&mut self, nodo: &Nodo) {
        self.analizar_condicion(nodo, "while");
        if let Some(cuerpo) = Self::hijo(nodo, "Cuerpo") {
            self.abrir_bloque("while");
            self.analizar_cuerpo(cuerpo);
            self.cerrar_bloque();
        }
    }

    fn analizar_for(&mut self, nodo: &Nodo) {
        let variable = Self::valor_de(nodo, "variable").unwrap_or_default();

        // El iterable define el tipo de la variable del bucle.
        let tipo_iterable = match Self::hijo(nodo, "iterable") {
            Some(iterable) => self.tipo_expresion(iterable),
            None => TIPO_ERROR.to_string(),
        };
        let tipo_variable = match tipo_iterable.as_str() {
            "range" => "int".to_string(),   // for i in range(n) -> i es int
            "str" => "str".to_string(),     // for c in "abc"    -> c es str
            TIPO_ERROR => TIPO_ERROR.to_string(),
            otro => {
                self.error(format!(
                    "Tipos inválidos: el 'for' necesita un iterable (range o str), \
                     se obtuvo '{}'",
                    otro
                ));
                TIPO_ERROR.to_string()
            }
        };

        // La variable del bucle vive solo dentro del bloque del for.
        self.abrir_bloque("for");
        let simbolo = Simbolo::variable(variable, tipo_variable, self.ambito_actual(), true);
        self.tabla.declarar(simbolo);
        if let Some(cuerpo) = Self::hijo(nodo, "Cuerpo") {
            self.analizar_cuerpo(cuerpo);
        }
        self.cerrar_bloque();
    }

    fn analizar_return(&mut self, nodo: &Nodo) {
        let funcion = match self.funcion_actual.clone() {
            Some(f) => f,
            None => {
                self.error("Código fuera de funciones: 'return' fuera de una función");
                return;
            }
        };
        self.tiene_return = true;

        match Self::hijo(nodo, "valor") {
            Some(valor) => {
                let tipo_valor = self.tipo_expresion(valor);
                if self.retorno_actual == "Void" {
                    self.error(format!(
                        "Tipos inválidos: la función '{}' es Void y no puede retornar un valor",
                        funcion
                    ));
                } else if !Self::compatibles(&self.retorno_actual.clone(), &tipo_valor) {
                    self.error(format!(
                        "Tipos inválidos: la función '{}' debe retornar '{}', se obtuvo '{}'",
                        funcion, self.retorno_actual, tipo_valor
                    ));
                }
            }
            None => {
                if self.retorno_actual != "Void" {
                    self.error(format!(
                        "Tipos inválidos: la función '{}' debe retornar un valor de tipo '{}'",
                        funcion, self.retorno_actual
                    ));
                }
            }
        }
    }

    // --- Llamadas (caso 7: Argumentos) -------------------------------------

    /// Analiza una llamada y devuelve el tipo de retorno de la función.
    fn analizar_llamada(&mut self, nodo: &Nodo) -> String {
        let nombre = Self::valor_de(nodo, "nombre").unwrap_or_default();

        // Primero se evalúan los argumentos, en orden de izquierda a derecha.
        let mut tipos_args: Vec<String> = Vec::new();
        if let Some(args) = Self::hijo(nodo, "Args") {
            for arg in &args.children {
                tipos_args.push(self.tipo_expresion(arg));
            }
        }

        // La función debe existir en la tabla (declarada antes de usarse).
        let info = self
            .tabla
            .resolver_mut(&nombre, &self.ambitos)
            .map(|s| {
                s.usado = true;
                (s.categoria.clone(), s.tipo.clone(), s.parametros.clone(), s.variadica)
            });

        let (categoria, retorno, parametros, variadica) = match info {
            Some(datos) => datos,
            None => {
                self.error(format!(
                    "Datos fuera de bloque: la función '{}' no está declarada",
                    nombre
                ));
                return TIPO_ERROR.to_string();
            }
        };

        if categoria != Categoria::Funcion {
            self.error(format!(
                "Tipos inválidos: '{}' es una {} y no se puede llamar como función",
                nombre,
                categoria.to_string().to_lowercase()
            ));
            return TIPO_ERROR.to_string();
        }

        if variadica {
            // print(...) acepta cualquier cantidad, pero cada argumento
            // debe producir un valor (no sirve pasar una llamada Void).
            for (i, tipo_arg) in tipos_args.iter().enumerate() {
                if tipo_arg == "Void" {
                    self.error(format!(
                        "Argumentos: el argumento {} de '{}' no produce ningún valor",
                        i + 1,
                        nombre
                    ));
                }
            }
        } else {
            // Cantidad exacta de argumentos...
            if tipos_args.len() != parametros.len() {
                self.error(format!(
                    "Argumentos: la función '{}' espera {} argumento(s) y recibe {}",
                    nombre,
                    parametros.len(),
                    tipos_args.len()
                ));
            }
            // ...y tipo compatible con cada parámetro declarado.
            for (i, ((p_nombre, p_tipo), tipo_arg)) in
                parametros.iter().zip(tipos_args.iter()).enumerate()
            {
                if !Self::compatibles(p_tipo, tipo_arg) {
                    self.error(format!(
                        "Argumentos: el argumento {} de '{}' ('{}') debe ser '{}', \
                         se obtuvo '{}'",
                        i + 1,
                        nombre,
                        p_nombre,
                        p_tipo,
                        tipo_arg
                    ));
                }
            }
        }

        retorno
    }

    // --- Tipado de expresiones ---------------------------------------------

    /// Recorre una expresión (izquierda a derecha) y devuelve su tipo:
    /// "int", "float", "str", "bool", "None", "range", "Void" o TIPO_ERROR.
    fn tipo_expresion(&mut self, nodo: &Nodo) -> String {
        let etiqueta = nodo.value.as_str();

        // Nodos envoltorio del parser: el tipo es el de su contenido.
        if matches!(etiqueta, "Expresión" | "valor" | "Condición" | "iterable") {
            return match nodo.children.first() {
                Some(hijo) => self.tipo_expresion(hijo),
                None => TIPO_ERROR.to_string(),
            };
        }

        if etiqueta == "Llamada" {
            return self.analizar_llamada(nodo);
        }

        if etiqueta == "<error>" {
            return TIPO_ERROR.to_string();
        }

        // Operadores (siempre tienen hijos). Las hojas se clasifican aparte.
        if !nodo.children.is_empty() {
            match etiqueta {
                "and" | "or" => {
                    let izq = self.tipo_expresion(&nodo.children[0]);
                    let der = self.tipo_expresion(&nodo.children[1]);
                    return self.tipo_binario(etiqueta, &izq, &der);
                }
                "not" => {
                    let operando = self.tipo_expresion(&nodo.children[0]);
                    if operando != "bool" && operando != TIPO_ERROR {
                        self.error(format!(
                            "Tipos inválidos: 'not' necesita un bool, se obtuvo '{}'",
                            operando
                        ));
                        return TIPO_ERROR.to_string();
                    }
                    return "bool".to_string();
                }
                "- (unario)" | "+ (unario)" => {
                    let operando = self.tipo_expresion(&nodo.children[0]);
                    if operando == TIPO_ERROR {
                        return TIPO_ERROR.to_string();
                    }
                    if !Self::es_numerico(&operando) {
                        self.error(format!(
                            "Tipos inválidos: el signo unario necesita un número, \
                             se obtuvo '{}'",
                            operando
                        ));
                        return TIPO_ERROR.to_string();
                    }
                    return operando;
                }
                op if nodo.children.len() == 2 => {
                    let izq = self.tipo_expresion(&nodo.children[0]);
                    let der = self.tipo_expresion(&nodo.children[1]);
                    return self.tipo_binario(op, &izq, &der);
                }
                _ => return TIPO_ERROR.to_string(),
            }
        }

        self.tipo_hoja(etiqueta.to_string())
    }

    /// Clasifica una hoja del árbol: literal o identificador.
    fn tipo_hoja(&mut self, lexema: String) -> String {
        if lexema.starts_with('"') || lexema.starts_with('\'') {
            return "str".to_string();
        }
        if lexema == "True" || lexema == "False" {
            return "bool".to_string();
        }
        if lexema == "None" {
            return "None".to_string();
        }
        if lexema.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            let minuscula = lexema.to_lowercase();
            if minuscula.starts_with("0x") || minuscula.starts_with("0o") || minuscula.starts_with("0b") {
                return "int".to_string();
            }
            if minuscula.contains('.') || minuscula.contains('e') {
                return "float".to_string();
            }
            return "int".to_string();
        }

        // Identificador: debe resolver en algún ámbito activo (caso 3).
        let encontrado = self
            .tabla
            .resolver_mut(&lexema, &self.ambitos)
            .map(|s| {
                s.usado = true;
                (s.categoria.clone(), s.tipo.clone(), s.inicializado)
            });
        match encontrado {
            Some((Categoria::Funcion, _, _)) => {
                self.error(format!(
                    "Tipos inválidos: '{}' es una función; para llamarla escribe '{}(...)'",
                    lexema, lexema
                ));
                TIPO_ERROR.to_string()
            }
            Some((_, tipo, inicializado)) => {
                if !inicializado {
                    self.advertencia(format!(
                        "La variable '{}' se usa sin haber sido inicializada",
                        lexema
                    ));
                }
                tipo
            }
            None => {
                self.error(format!(
                    "Datos fuera de bloque: la variable '{}' no está declarada en \
                     ningún ámbito activo",
                    lexema
                ));
                TIPO_ERROR.to_string()
            }
        }
    }

    /// Tipo resultante de `izq op der`; reporta el error si no son operables.
    fn tipo_binario(&mut self, op: &str, izq: &str, der: &str) -> String {
        // Si un operando ya falló, no se reporta otra vez (evita cascada).
        if izq == TIPO_ERROR || der == TIPO_ERROR {
            return TIPO_ERROR.to_string();
        }

        let ambos_numericos = Self::es_numerico(izq) && Self::es_numerico(der);
        let promocion = if izq == "float" || der == "float" { "float" } else { "int" };

        let resultado = match op {
            "+" => {
                if izq == "str" && der == "str" {
                    Some("str")
                } else if ambos_numericos {
                    Some(promocion)
                } else {
                    None
                }
            }
            "-" | "*" | "%" | "**" => ambos_numericos.then_some(promocion),
            "/" => ambos_numericos.then_some("float"),
            "//" => ambos_numericos.then_some(promocion),
            "<" | ">" | "<=" | ">=" => {
                (ambos_numericos || (izq == "str" && der == "str")).then_some("bool")
            }
            "==" | "!=" => (ambos_numericos || izq == der).then_some("bool"),
            "and" | "or" => (izq == "bool" && der == "bool").then_some("bool"),
            _ => None,
        };

        match resultado {
            Some(tipo) => tipo.to_string(),
            None => {
                self.error(format!(
                    "Tipos inválidos: no se puede operar '{}' {} '{}'",
                    izq, op, der
                ));
                TIPO_ERROR.to_string()
            }
        }
    }

    // --- Reglas de tipos ---------------------------------------------------

    fn es_numerico(tipo: &str) -> bool {
        tipo == "int" || tipo == "float"
    }

    /// ¿Un valor de `tipo_valor` puede guardarse donde se espera `esperado`?
    /// Se acepta el mismo tipo y la promoción int -> float.
    fn compatibles(esperado: &str, tipo_valor: &str) -> bool {
        if esperado == TIPO_ERROR || tipo_valor == TIPO_ERROR {
            return true; // el error ya fue reportado donde se originó
        }
        esperado == tipo_valor || (esperado == "float" && tipo_valor == "int")
    }

    /// Caso 2: valida una anotación de tipo (int, float, str, bool).
    fn validar_tipo(&mut self, tipo: &str, contexto: &str) -> bool {
        if TIPOS_VALIDOS.contains(&tipo) {
            return true;
        }
        if TIPOS_ARRAY.contains(&tipo) {
            self.error(format!(
                "Tipos inválidos: los arrays/colecciones ('{}') no están soportados en {}",
                tipo, contexto
            ));
        } else {
            self.error(format!(
                "Tipos inválidos: el tipo '{}' de {} no existe en el lenguaje",
                tipo, contexto
            ));
        }
        false
    }
}
