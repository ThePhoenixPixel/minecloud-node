use crate::cloud::Cloud;

pub mod core {

    pub mod group;
    pub mod task;
    pub mod template;
    //pub mod node;
    pub mod installer;

    pub mod service;
    pub mod services_all;
    pub mod services_local;
    pub mod services_network;
    pub mod software;
}

pub mod database {
    pub mod database_manger;
    pub mod db_tools;
    pub mod db_treiber {
        pub mod mysql;
        pub mod sqlite;
    }

    pub mod table {
        pub mod table_players;
        pub mod table_player_sessions;
        pub mod table_player_events;
        pub mod table_service_events;
        pub mod table_services;
    }
}

pub mod sys_config {
    pub mod cloud_config;
    pub mod software_config;
}

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

const VERSION: &str = "0.1";

#[tokio::main]
async fn main() {
    println!("Start MineCloud...");

    // Cloud starten
    Cloud::enable(VERSION).await;

    println!("MineCloud Stop");
    println!("Goodbye");
}
