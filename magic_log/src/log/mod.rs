pub mod macros;
pub mod store;

pub use store::{LogEntry, LogLevel, LogStore, global_store};

static LOG_CALLBACK: std::sync::OnceLock<fn(LogLevel, &str, &str)> = std::sync::OnceLock::new();

pub fn set_log_callback(cb: fn(LogLevel, &str, &str)) {
    let _ = LOG_CALLBACK.set(cb);
}

pub fn log_callback() -> Option<fn(LogLevel, &str, &str)> {
    LOG_CALLBACK.get().copied()
}

pub fn push_log(level: LogLevel, source: &str, msg: String) {
    if let Some(cb) = LOG_CALLBACK.get().copied() {
        cb(level, source, &msg);
    } else {
        let entry = LogEntry::new(level, msg, source);
        if let Ok(mut store) = global_store().lock() {
            store.push(entry);
        }
    }
}
