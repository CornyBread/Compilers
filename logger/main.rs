#[path = "mod.rs"]
mod logger;

use logger::{LogEntry, LogLevel, Logger};

fn main() -> std::io::Result<()> {
    let mut logger = Logger::named("demo");
    let mut scratch_logger = Logger::new();

    scratch_logger.info("Logger auxiliar creado con new");
    scratch_logger.clear();
`
    logger.log(LogLevel::Info, "Logger iniciado");
    logger.debug("Preparando datos para depuracion");
    logger.warn("Esto es un aviso de ejemplo");
    logger.error("Esto es un error de ejemplo");

    logger.save_to_file("logs.txt")?;

    let last_entry: Option<&LogEntry> = logger.entries().last();
    println!("Ultima entrada: {:?}", last_entry);

    for entry in logger.entries() {
        println!("{}", entry);
    }

    Ok(())
}