use crate::cloud::Cloud;

pub mod types;
pub mod config;

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

pub mod utils {
    pub mod error;
    pub mod error_kind;
    pub mod file_inlude;
    pub mod log;
    pub mod logger;
    pub mod player;
    pub mod player_action;
    pub mod server_type;
    pub mod service_status;
    pub mod utils;
    #[macro_use]
    pub mod logger_macros;
}

pub mod cloud;

pub mod language;
pub mod api;
pub mod database;

const VERSION: &str = "0.1";

#[tokio::main]
async fn main() {
    println!("Start MineCloud...");

    // Cloud starten
    Cloud::enable(VERSION).await;

    println!("MineCloud Stop");
    println!("Goodbye");
}
