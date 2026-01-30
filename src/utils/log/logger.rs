use chrono::Local;
use colored::{ColoredString, Colorize};
use std::fs::OpenOptions;
use std::io::Write;
use std::{env, fs};
use once_cell::sync::OnceCell;

use crate::config::cloud_config::CloudConfig;
use crate::utils::log::LogType;

pub static LOG_LEVEL: OnceCell<u8> = OnceCell::new();


pub struct Logger;

impl Logger {

    pub fn init_log_level() {
        let _ = LOG_LEVEL.set(CloudConfig::get().get_log_level());
    }

    pub fn get_log_level() -> u8 {
        *LOG_LEVEL.get().unwrap_or(&9)
    }

    fn log(args: std::fmt::Arguments, log_type: LogType) {
        let msg = format!(
            "{}",
            format_args!(
                "{} {} {} {}",
                ColoredString::from(CloudConfig::get().get_prefix()).blue(),
                log_type.to_string_colored(),
                ColoredString::from(">>").blue(),
                args
            )
        );

        // print the args in the cmd
        println!("{}", &msg);

        // write the cmd output in the log file
        Logger::write_in_file(msg);
    }

    pub fn info(args: std::fmt::Arguments) {
        Logger::log(args, LogType::Info);
    }

    pub fn warning(args: std::fmt::Arguments) {
        Logger::log(args, LogType::Warning);
    }

    pub fn error(args: std::fmt::Arguments) {
        Logger::log(args, LogType::Error);
    }

    fn write_in_file(msg: String) {
        let mut log_path =
            env::current_exe().expect("Cloud Error can not get the exe path of the cloud system");
        log_path.pop();
        log_path.push("log");
        fs::create_dir_all(&log_path).expect("Cant create Log File path in 'write_in_file'");

        let file_name = format!("log_{}.log", Local::now().format("%Y-%m-%d"));
        log_path.push(&file_name);

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("Log system has an error and cannot create the log file");

        if writeln!(file, "{}", msg).is_err() {
            eprintln!("Log System has an Error");
        }
    }
}
