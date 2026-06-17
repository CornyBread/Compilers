// Definimos un Trait (Interfaz). Actúa como un contrato que obliga a 
// cualquier estructura que lo implemente a tener el método 'print_structure'.
pub trait Printable {
    fn print_structure(&self);
}