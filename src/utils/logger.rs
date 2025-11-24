use chrono::Local;
use colored::{ColoredString, Colorize};
use std::fs::OpenOptions;
use std::io::Write;
use std::{env, fs};

use crate::sys_config::cloud_config::CloudConfig;
use crate::utils::log::Log;

pub struct Logger;

impl Logger {
    fn log(args: std::fmt::Arguments, log_level: Log) {
        let msg = format!(
            "{}",
            format_args!(
                "{} {} {} {}",
                ColoredString::from(CloudConfig::get().get_prefix()).blue(),
                Log::get(log_level).to_string(),
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
        Logger::log(args, Log::Info);
    }

    pub fn warning(args: std::fmt::Arguments) {
        Logger::log(args, Log::Warning);
    }

    pub fn error(args: std::fmt::Arguments) {
        Logger::log(args, Log::Error);
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
