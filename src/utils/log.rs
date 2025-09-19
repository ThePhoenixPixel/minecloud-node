use colored::{ColoredString, Colorize};

pub enum Log {
    Info,
    Warning,
    Error,
}

impl Log {
    pub fn get(log: Log) -> ColoredString {
        return match log {
            Log::Info => ColoredString::from("[info]").green(),
            Log::Warning => ColoredString::from("[warning]").yellow(),
            Log::Error => ColoredString::from("[error]").red(),
        };
    }
}
