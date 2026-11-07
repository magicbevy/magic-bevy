#[macro_export]
macro_rules! oracle_info {
    ($source:expr, $msg:expr) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Info, $source, $msg.into());
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Info, $source, format!($fmt, $($arg)*));
        }
    };
}

#[macro_export]
macro_rules! oracle_warn {
    ($source:expr, $msg:expr) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Warn, $source, $msg.into());
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Warn, $source, format!($fmt, $($arg)*));
        }
    };
}

#[macro_export]
macro_rules! oracle_error {
    ($source:expr, $msg:expr) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Error, $source, $msg.into());
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Error, $source, format!($fmt, $($arg)*));
        }
    };
}

#[macro_export]
macro_rules! oracle_success {
    ($source:expr, $msg:expr) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Success, $source, $msg.into());
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Success, $source, format!($fmt, $($arg)*));
        }
    };
}

#[macro_export]
macro_rules! oracle_debug {
    ($source:expr, $msg:expr) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Debug, $source, $msg.into());
        }
    };
    ($source:expr, $fmt:expr, $($arg:tt)*) => {
        {
            $crate::log::push_log($crate::log::LogLevel::Debug, $source, format!($fmt, $($arg)*));
        }
    };
}
