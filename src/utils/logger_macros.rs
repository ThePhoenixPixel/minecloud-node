#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => ({
        Logger::info(format_args!($($arg)*));
    });
}
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => ({
        Logger::warning(format_args!($($arg)*));
    });
}
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => ({
        Logger::error(format_args!($($arg)*));
    });
}
