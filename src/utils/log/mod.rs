use colored::{ColoredString, Colorize};

pub mod logger;

#[macro_use]
pub mod macros;


pub enum LogType {
    Info,
    Warning,
    Error,
}

impl LogType {
    pub fn to_string_colored(&self) -> ColoredString {
        match self {
            LogType::Info => ColoredString::from("[info]").green(),
            LogType::Warning => ColoredString::from("[warning]").yellow(),
            LogType::Error => ColoredString::from("[error]").red(),
        }
    }
}


