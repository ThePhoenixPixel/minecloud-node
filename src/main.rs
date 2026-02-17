use std::io::Write;
use std::fs::OpenOptions;
use chrono::Local;
use database_manager::config::{DBMysqlConfig, DatabaseConfig};
use crate::cloud::Cloud;
use crate::utils::utils::Utils;

pub mod types;
pub mod config;
pub mod utils;
pub mod cloud;
pub mod language;
pub mod api;
pub mod database;
pub mod manager;
pub mod node;

pub mod terminal {
    pub mod cmd;
    pub mod command_manager;

    pub mod command {
        pub mod cmd_help;
        pub mod cmd_me;
        pub mod cmd_service;
        pub mod cmd_task;
        pub mod cmd_template;
    }
}

const VERSION: &str = "0.1";

#[tokio::main]
async fn main() {
    println!("Start MineCloud...");

    // Cloud starten
    Cloud::enable(VERSION).await;

    println!("MineCloud Stop");
    println!("Goodbye");
}
