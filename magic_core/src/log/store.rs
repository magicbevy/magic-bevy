use std::sync::OnceLock;

use chrono::Local;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Success,
    Debug,
}

impl LogLevel {
    pub fn tag(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Success => "SUCCESS",
            LogLevel::Debug => "DEBUG",
        }
    }

    pub fn color_code(&self) -> egui::Color32 {
        match self {
            LogLevel::Info => egui::Color32::from_rgb(100, 200, 255),
            LogLevel::Warn => egui::Color32::from_rgb(255, 200, 50),
            LogLevel::Error => egui::Color32::from_rgb(255, 80, 80),
            LogLevel::Success => egui::Color32::from_rgb(100, 255, 100),
            LogLevel::Debug => egui::Color32::from_rgb(200, 200, 200),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub timestamp: chrono::DateTime<Local>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            source: source.into(),
            timestamp: Local::now(),
        }
    }
}

pub struct LogStore {
    entries: Vec<LogEntry>,
    max_entries: usize,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 5000,
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

static LOG_STORE: OnceLock<std::sync::Mutex<LogStore>> = OnceLock::new();

pub fn global_store() -> &'static std::sync::Mutex<LogStore> {
    LOG_STORE.get_or_init(|| std::sync::Mutex::new(LogStore::new()))
}
