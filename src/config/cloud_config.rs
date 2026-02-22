use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use bx::network::address::Address;
use bx::network::url::Url;
use database_manager::config::DatabaseConfig;

use crate::cloud::Cloud;
use crate::utils::utils::Utils;
use crate::{log_error, log_info};

#[derive(Serialize, Deserialize, Clone)]
pub struct CloudConfig {
    name: String,
    prefix: String,
    language: String,
    log_level: u8,
    server_host: String,
    max_ram: u64,
    node_host: Address,
    rest_api: Address,
    database: DatabaseConfig,
    path: CloudConfigPath,
}

impl CloudConfig {
    pub fn new(
        name: &String,
        prefix: &String,
        language: &String,
        log_level: &u8,
        server_host: &String,
        max_ram: &u64,
        node_host: &Address,
        rest_api: &Address,
        datenbank: &DatabaseConfig,
        path: &CloudConfigPath,
    ) -> CloudConfig {
        CloudConfig {
            name: name.clone(),
            prefix: prefix.clone(),
            language: language.clone(),
            log_level: log_level.clone(),
            server_host: server_host.clone(),
            max_ram: max_ram.clone(),
            node_host: node_host.clone(),
            rest_api: rest_api.clone(),
            database: datenbank.clone(),
            path: path.clone(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_prefix(&self) -> String {
        self.prefix.clone()
    }

    pub fn get_language(&self) -> String {
        self.language.clone()
    }

    pub fn get_log_level(&self) -> u8 {
        self.log_level
    }
    pub fn get_server_host(&self) -> String {
        self.server_host.clone()
    }

    pub fn get_max_ram(&self) -> u64 {
        self.max_ram.clone()
    }

    pub fn get_node_host(&self) -> Address {
        self.node_host.clone()
    }

    pub fn get_rest_api(&self) -> Address {
        self.rest_api.clone()
    }

    pub fn get_db_config(&self) -> DatabaseConfig {
        self.database.clone()
    }
    pub fn get_cloud_path(&self) -> CloudConfigPath {
        self.path.clone()
    }

    pub async fn check_and_get(url: &String) -> CloudConfig {
        if !Cloud::get_working_path().join("config.json").exists() {
            CloudConfig::install(url).await;
        }
        CloudConfig::get()
    }

    pub async fn install(start_url: &String) {
        // get the response from the download
        let url = format!("{}/config.json", start_url);
        match Url::download_file(url.as_str(), &Cloud::get_working_path().join("config.json")).await
        {
            Ok(_) => log_info!("Successfully download the Software Config from {}", url),
            Err(e) => {
                log_error!("{}", &e.to_string());
                panic!("MineCloud has an fatal Error");
            }
        }
    }

    pub fn get() -> CloudConfig {
        let path = Cloud::get_working_path().join("config.json");
        // Versuche, den Inhalt der Datei zu lesen
        let file_content = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("{}", &e.to_string());
            get_default_file()
        });

        // Versuche, den Inhalt der Datei zu deserialisieren
        match serde_json::from_str(&file_content) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Fehler beim Deserialisieren der Config");
                eprintln!("{}", &e.to_string());
                panic!("The GameCloud has a fatal Error");
            }
        }
    }

    pub fn print(&self) {
        println!("CloudConfig:");
        println!("Name: {}", self.get_name());
        println!("Prefix: {}", self.get_prefix());
        println!("Language: {}", self.get_language());
        println!("Server Host: {}", self.get_server_host());
        println!("Max RAM: {}", self.get_max_ram());
        println!("Node Host IP: {}", self.get_node_host().get_ip());
        println!("Node Host PORT: {}", self.get_node_host().get_port());
        println!("REST API IP: {}", self.get_rest_api().get_ip());
        println!("REST API PORT: {}", self.get_rest_api().get_port());

        let path = self.get_cloud_path();
        println!("Path:");
        println!("  Task Folder: {}", path.get_task_folder());
        println!("  Template Folder: {}", path.get_template_folder());
        println!("  Group Folder: {}", path.get_group_folder());

        let service_folder = path.get_service_folder();
        println!("  Service Folder:");
        println!("    Temp Folder: {}", service_folder.get_temp_folder());
        println!("    Static Folder: {}", service_folder.get_static_folder());

        let system_folder = path.get_system_folder();
        println!("  System Folder:");
        println!(
            "    Software Config: {}",
            system_folder.get_software_config()
        );
        println!("    Default Task: {}", system_folder.get_default_task());
        println!(
            "    System Plugins Folder: {}",
            system_folder.get_system_plugins_folder()
        );
        println!(
            "    Software Files Folder: {}",
            system_folder.get_software_files_folder()
        );
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CloudConfigPath {
    task_folder: String,
    template_folder: String,
    group_folder: String,
    service_folder: CloudConfigService,
    system_folder: CloudConfigSystem,
}

impl CloudConfigPath {
    pub fn new(
        task_folder: &String,
        template_folder: &String,
        group_folder: &String,
        service_folder: &CloudConfigService,
        system_folder: &CloudConfigSystem,
    ) -> CloudConfigPath {
        CloudConfigPath {
            task_folder: task_folder.clone(),
            template_folder: template_folder.clone(),
            group_folder: group_folder.clone(),
            service_folder: service_folder.clone(),
            system_folder: system_folder.clone(),
        }
    }

    pub fn get_task_folder(&self) -> String {
        self.task_folder.clone()
    }

    pub fn get_task_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.task_folder)
    }

    pub fn get_template_folder(&self) -> String {
        self.template_folder.clone()
    }
    pub fn get_template_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.template_folder)
    }

    pub fn get_group_folder(&self) -> String {
        self.group_folder.clone()
    }
    pub fn get_group_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.group_folder)
    }

    pub fn get_service_folder(&self) -> CloudConfigService {
        self.service_folder.clone()
    }

    pub fn get_system_folder(&self) -> CloudConfigSystem {
        self.system_folder.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CloudConfigService {
    temp_folder: String,
    static_folder: String,
}

impl CloudConfigService {
    pub fn new(temp_folder: &String, static_folder: &String) -> CloudConfigService {
        CloudConfigService {
            temp_folder: temp_folder.clone(),
            static_folder: static_folder.clone(),
        }
    }

    pub fn get_temp_folder(&self) -> String {
        self.temp_folder.clone()
    }

    pub fn get_temp_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.temp_folder)
    }

    pub fn get_static_folder(&self) -> String {
        self.static_folder.clone()
    }

    pub fn get_static_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.static_folder)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CloudConfigSystem {
    software_config: String,
    default_task: String,
    system_plugins_folder: String,
    software_files_folder: String,
    software_lib_folder: String,
}

impl CloudConfigSystem {
    pub fn new(
        software_config: &String,
        default_task: &String,
        system_plugins_folder: &String,
        software_files_folder: &String,
        software_lib_folder: &String,
    ) -> CloudConfigSystem {
        CloudConfigSystem {
            software_config: software_config.clone(),
            default_task: default_task.clone(),
            system_plugins_folder: system_plugins_folder.clone(),
            software_files_folder: software_files_folder.clone(),
            software_lib_folder: software_lib_folder.clone(),
        }
    }

    pub fn get_software_config(&self) -> String {
        format!("{}software.json", self.software_config)
    }

    pub fn get_software_config_path(&self) -> PathBuf {
        Utils::get_path(&self.software_config).join("software.json")
    }

    pub fn get_default_task(&self) -> String {
        self.default_task.clone()
    }

    pub fn get_default_task_path(&self) -> PathBuf {
        Utils::get_path(&self.default_task)
    }
    pub fn get_system_plugins_folder(&self) -> String {
        self.system_plugins_folder.clone()
    }

    pub fn get_system_plugins_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.system_plugins_folder)
    }
    pub fn get_software_files_folder(&self) -> String {
        self.software_files_folder.clone()
    }

    pub fn get_software_files_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.software_files_folder)
    }
    pub fn get_software_lib_folder(&self) -> String {
        self.software_lib_folder.clone()
    }
    pub fn get_software_lib_folder_path(&self) -> PathBuf {
        Utils::get_path(&self.get_software_lib_folder())
    }
}

fn get_default_file() -> String {
    let json_str = r#"
    {
      "name": "Node-1",
      "prefix": "[Game Cloud]",
      "language": "de",
      "server_host": "127.0.0.1",
      "max_ram": 2028,
      "database": {
        "typ": "SQLITE",
        "sqlite": {
          "file": "~database.db"
        },
        "mysql": {
          "host": {
            "ip": "127.0.0.1",
            "port": 3306
          },
          "username": "minecloud",
          "password": "minecloud123",
          "database": "minecloud"
        }
      },
      "node_host": {
        "ip": "127.0.0.1",
        "port": 5050
      },
      "rest_api": {
        "ip": "127.0.0.1",
        "port": 5051
      },
      "path": {
        "task_folder": "~task",
        "template_folder": "~template",
        "service_folder": {
          "temp_folder": "~service/temp",
          "static_folder": "~service/static"
        },
        "system_folder": {
          "software_config": "~config",
          "default_task": "~config",
          "system_plugins_folder": "~config/system_plugins",
          "software_files_folder": "~config/software_files"
        }
      }
    }
    "#;
    json_str.to_string()
}
