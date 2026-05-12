use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
	Debug,
	Info,
	Warn,
	Error,
}

impl fmt::Display for LogLevel {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let label = match self {
			LogLevel::Debug => "DEBUG",
			LogLevel::Info => "INFO",
			LogLevel::Warn => "WARN",
			LogLevel::Error => "ERROR",
		};

		write!(f, "{}", label)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
	pub timestamp: u64,
	pub logger_name: Option<String>,
	pub level: LogLevel,
	pub message: String,
}

impl fmt::Display for LogEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.logger_name {
			Some(name) => write!(f, "[{}][{}][{}] {}", self.timestamp, name, self.level, self.message),
			None => write!(f, "[{}][{}] {}", self.timestamp, self.level, self.message),
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct Logger {
	name: Option<String>,
	entries: Vec<LogEntry>,
}

impl Logger {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn named(name: impl Into<String>) -> Self {
		Self {
			name: Some(name.into()),
			entries: Vec::new(),
		}
	}

	pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
		self.entries.push(LogEntry {
			timestamp: Self::current_timestamp(),
			logger_name: self.name.clone(),
			level,
			message: message.into(),
		});
	}

	pub fn debug(&mut self, message: impl Into<String>) {
		self.log(LogLevel::Debug, message);
	}

	pub fn info(&mut self, message: impl Into<String>) {
		self.log(LogLevel::Info, message);
	}

	pub fn warn(&mut self, message: impl Into<String>) {
		self.log(LogLevel::Warn, message);
	}

	pub fn error(&mut self, message: impl Into<String>) {
		self.log(LogLevel::Error, message);
	}

	pub fn entries(&self) -> &[LogEntry] {
		&self.entries
	}

	pub fn clear(&mut self) {
		self.entries.clear();
	}

	pub fn save_to_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
		let mut file = File::create(path)?;

		for entry in &self.entries {
			writeln!(file, "{}", entry)?;
		}

		Ok(())
	}

	fn current_timestamp() -> u64 {
		SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map(|duration| duration.as_secs())
			.unwrap_or_default()
	}
}
