#[macro_export]
macro_rules! oracle_info {
    ($source:expr, $msg:expr) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Info,
                $msg,
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Info,
                format!($fmt, $($arg)*),
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
}

#[macro_export]
macro_rules! oracle_warn {
    ($source:expr, $msg:expr) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Warn,
                $msg,
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Warn,
                format!($fmt, $($arg)*),
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
}

#[macro_export]
macro_rules! oracle_error {
    ($source:expr, $msg:expr) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Error,
                $msg,
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Error,
                format!($fmt, $($arg)*),
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
}

#[macro_export]
macro_rules! oracle_success {
    ($source:expr, $msg:expr) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Success,
                $msg,
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Success,
                format!($fmt, $($arg)*),
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
}

#[macro_export]
macro_rules! oracle_debug {
    ($source:expr, $msg:expr) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Debug,
                $msg,
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            use $crate::log::global_store;
            let entry = $crate::log::LogEntry::new(
                $crate::log::LogLevel::Debug,
                format!($fmt, $($arg)*),
                $source,
            );
            if let Ok(mut store) = global_store().lock() {
                store.push(entry);
            }
        }
    };
}
