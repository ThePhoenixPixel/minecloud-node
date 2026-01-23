#[macro_export]
macro_rules! log_info {
    // mit Level
    ($lvl:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $lvl <= crate::utils::logger::Logger::get_log_level() {
            Logger::info(format_args!($fmt $(, $arg)*));
        }
    }};

    // ohne Level (Default-Level = 3)
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        if 3 <= crate::utils::logger::Logger::get_log_level() {
            Logger::info(format_args!($fmt $(, $arg)*));
        }
    }};
}

#[macro_export]
macro_rules! log_warning {
    // mit Level
    ($lvl:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $lvl <= crate::utils::logger::Logger::get_log_level() {
            Logger::warning(format_args!($fmt $(, $arg)*));
        }
    }};

    // ohne Level (Default-Level = 2)
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        if 2 <= crate::utils::logger::Logger::get_log_level() {
            Logger::warning(format_args!($fmt $(, $arg)*));
        }
    }};
}

#[macro_export]
macro_rules! log_error {
    // mit Level
    ($lvl:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $lvl <= crate::utils::logger::Logger::get_log_level() {
            Logger::error(format_args!($fmt $(, $arg)*));
        }
    }};

    // ohne Level (Default-Level = 1)
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        if 1 <= crate::utils::logger::Logger::get_log_level() {
            Logger::error(format_args!($fmt $(, $arg)*));
        }
    }};
}
