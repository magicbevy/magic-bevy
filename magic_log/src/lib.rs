pub mod log;
pub mod oracle;
#[macro_use]
pub mod logger;

pub use colored;
pub use log::{
    LogEntry, LogLevel, LogStore, global_store, log_callback, push_log, set_log_callback,
};
