#[macro_export]
macro_rules! log_info {
    // mit Level
    ($lvl:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $lvl <= crate::utils::log::logger::Logger::get_log_level() {
            crate::utils::log::logger::Logger::info(format_args!($fmt $(, $arg)*));
        }
    }};

    // ohne Level (Default-Level = 3)
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        if 3 <= crate::utils::log::logger::Logger::get_log_level() {
            crate::utils::log::logger::Logger::info(format_args!($fmt $(, $arg)*));
        }
    }};
}

#[macro_export]
macro_rules! log_warning {
    // mit Level
    ($lvl:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $lvl <= crate::utils::log::logger::Logger::get_log_level() {
            crate::utils::log::logger::Logger::warning(format_args!($fmt $(, $arg)*));
        }
    }};

    // ohne Level (Default-Level = 2)
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        if 2 <= crate::utils::log::logger::Logger::get_log_level() {
            crate::utils::log::logger::Logger::warning(format_args!($fmt $(, $arg)*));
        }
    }};
}

#[macro_export]
macro_rules! log_error {
    // mit Level
    ($lvl:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $lvl <= crate::utils::log::logger::Logger::get_log_level() {
            crate::utils::log::logger::Logger::error(format_args!($fmt $(, $arg)*));
        }
    }};

    // ohne Level (Default-Level = 1)
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        if 1 <= crate::utils::log::logger::Logger::get_log_level() {
            crate::utils::log::logger::Logger::error(format_args!($fmt $(, $arg)*));
        }
    }};
}
