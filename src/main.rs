use crate::cloud::Cloud;
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "rest-api")]
pub mod rest_api {
    pub mod restapi_main;
    pub mod restapi_service;
    pub mod restapi_task;
}

pub mod core {

    pub mod task;
    pub mod template;
    //pub mod group;
    //pub mod node;
    pub mod installer;

    pub mod service;
    pub mod services_all;
    pub mod services_local;
    pub mod services_network;
    pub mod software;
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

pub mod node_api {
    pub mod node_main;
    pub mod node_service;
}

pub mod utils {
    pub mod file_inlude;
    pub mod log;
    pub mod logger;
    pub mod server_type;
    pub mod error;
    pub mod error_kind;
    pub mod service_status;
    pub mod utils;
    #[macro_use]
    pub mod logger_macros;
}

pub mod cloud;

pub mod language;

const VERSION: &str = "0.1";

#[tokio::main]
async fn main() {
    println!("Start MineCloud...");

    // Cloud-Instanz erstellen
    let cloud = Arc::new(RwLock::new(Cloud::new()));

    // Cloud starten
    Cloud::enable(cloud, VERSION).await;

    println!("MineCloud Stop");
    println!("Goodbye");
}
