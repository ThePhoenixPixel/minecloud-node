use bx::network::url::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::cloud_config::CloudConfig;
use crate::types::ServerType;
use crate::{log_error, log_info, log_warning};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SoftwareConfig {
    software_type: HashMap<String, SoftwareType>,
}

impl SoftwareConfig {
    pub fn new(cloud_config: Arc<CloudConfig>) -> SoftwareConfig {
        let file_content = fs::read_to_string(
            CloudConfig::get()
                .get_cloud_path()
                .get_system_folder()
                .get_software_config_path(),
        )
        .unwrap_or_else(|e| {
            log_warning!("Please specify the correct path to the software file configuration");
            log_error!("{}", &e.to_string());
            get_default_software_config()
        });

        let mut config: SoftwareConfig = match serde_json::from_str(&file_content) {
            Ok(config) => config,
            Err(e) => {
                log_error!(1, "Error deserializing the software file configuration");
                log_error!(1, "{}", &e.to_string());
                panic!("The GameCloud has a fatal Error");
            }
        };

        for (type_name, software_type) in config.software_type.iter_mut() {
            software_type.set_type_for_software_names(type_name);
        }

        config
    }

    pub fn get() -> SoftwareConfig {
        let file_content = fs::read_to_string(
            CloudConfig::get()
                .get_cloud_path()
                .get_system_folder()
                .get_software_config_path(),
        )
        .unwrap_or_else(|e| {
            log_warning!("Please specify the correct path to the software file configuration");
            log_error!("{}", &e.to_string());
            get_default_software_config()
        });

        let mut config: SoftwareConfig = match serde_json::from_str(&file_content) {
            Ok(config) => config,
            Err(e) => {
                log_error!(1, "Error deserializing the software file configuration");
                log_error!(1, "{}", &e.to_string());
                panic!("The GameCloud has a fatal Error");
            }
        };

        for (type_name, software_type) in config.software_type.iter_mut() {
            software_type.set_type_for_software_names(type_name);
        }

        config
    }

    pub fn get_software_type(&self, software_type: &str) -> SoftwareType {
        self.software_type
            .get(&software_type.to_lowercase())
            .cloned()
            .unwrap_or_else(default_software_type)
    }

    pub fn get_software_types(&self) -> HashMap<String, SoftwareType> {
        self.software_type.clone()
    }

    pub fn remove_software_type(&mut self, name: &str) {
        self.software_type.remove(name);
    }

    pub async fn check(url: &String) {
        if !CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_config_path()
            .exists()
        {
            SoftwareConfig::install(url).await;
        }
    }

    pub async fn install(start_url: &String) {
        let url = format!("{}/config/software.json", start_url);
        let mut folder_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_config_path()
            .join("software.json");

        folder_path.pop();
        match Url::download_file(url.as_str(), &folder_path).await {
            Ok(_) => log_info!("Successfully download the Software Config from {}", url),
            Err(e) => {
                log_error!("{}", e);
                panic!("Game Cloud has an fatal Error");
            }
        }
    }

    pub fn get_software(software_type: &str, software_name: &str) -> SoftwareName {
        let software_config = SoftwareConfig::get();
        let software_type = software_config.get_software_type(software_type);
        software_type.get_software_name(software_name)
    }

    pub fn is_exists(typ: &str, name: &str) -> bool {
        let software_types = SoftwareConfig::get().get_software_types();

        for (software_typ, software_type) in software_types {
            if !(software_typ == typ.to_string()) {
                continue;
            }
            for software_name in software_type.get_software_names() {
                if software_name.get_name() == name.to_string() {
                    return true;
                }
            }
        }
        false
    }
}

// -----------------------------------------------------------
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SoftwareType {
    software_name: Vec<SoftwareName>,
}

impl SoftwareType {
    pub fn get_software_name(&self, name: &str) -> SoftwareName {
        self.software_name
            .iter()
            .find(|software| software.get_name() == name.to_lowercase())
            .cloned()
            .unwrap_or_else(default_software_name)
    }

    pub fn get_software_names(&self) -> Vec<SoftwareName> {
        self.software_name.clone()
    }

    pub fn remove_software_name(&mut self, software_name: &SoftwareName) {
        self.software_name
            .insert(self.software_name.len() + 1, software_name.clone());
    }
    pub fn get_type_name(&self) -> String {
        for (name, software_type) in SoftwareConfig::get().get_software_types() {
            if software_type.clone() == self.clone() {
                return name;
            }
        }
        String::new()
    }
    pub fn set_type_for_software_names(&mut self, type_name: &str) {
        for software_name in &mut self.software_name {
            software_name.set_type(type_name);
        }
    }
}

//-------------------------------------------------------------
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct SoftwareName {
    name: String,
    software_file: SoftwareFile,
    environment: Environment,
    max_ram: u32,
    ip_path: String,
    port_path: String,
    server_type: ServerType,
    system_plugin: SystemPlugin,
    software_lib: HashMap<String, String>,

    #[serde(skip)]
    typ: String,
}

impl SoftwareName {
    pub fn get_name(&self) -> String {
        self.name.to_lowercase()
    }

    pub fn get_software_file(&self) -> SoftwareFile {
        self.software_file.clone()
    }

    pub fn get_environment(&self) -> Environment {
        self.environment.clone()
    }
    pub fn get_max_ram(&self) -> u32 {
        self.max_ram.clone()
    }
    pub fn get_ip_path(&self) -> String {
        self.ip_path.clone()
    }

    pub fn get_port_path(&self) -> String {
        self.port_path.clone()
    }

    pub fn get_server_type(&self) -> ServerType {
        self.server_type.clone()
    }

    pub fn get_system_plugin(&self) -> SystemPlugin {
        self.system_plugin.clone()
    }

    pub fn get_software_file_path(&self) -> PathBuf {
        let software_files_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_files_folder_path();
        let software_path =
            match Url::extract_extension_from_url(&self.get_software_file().get_url()) {
                Some(ext) => software_files_path.join(format!("{}.{}", self.get_name(), ext)),
                None => software_files_path.join(self.get_name()),
            };

        software_path
    }
    pub fn get_software_lib_str(&self) -> HashMap<String, String> {
        self.software_lib.clone()
    }
    pub fn get_software_lib(&self) -> HashMap<String, PathBuf> {
        let mut map = HashMap::new();
        let software_lib_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_lib_folder_path();
        let software_type = match self.get_software_type() {
            Some(typ) => typ,
            None => return HashMap::new(),
        };

        for (url_str, path_str) in self.get_software_lib_str() {
            let path = software_lib_path
                .join(software_type.get_type_name())
                .join(self.get_name())
                .join(path_str);
            map.insert(url_str, path);
        }

        map
    }
    pub fn get_software_type(&self) -> Option<SoftwareType> {
        let software_types = SoftwareConfig::get().get_software_types();
        for (_name, software_type) in software_types {
            for software_name in software_type.get_software_names() {
                if self.get_name() == software_name.get_name() {
                    return Some(software_type.clone());
                }
            }
        }
        None
    }

    pub fn get_typ(&self) -> String {
        self.typ.clone()
    }
    pub fn set_type(&mut self, typ: &str) {
        self.typ = String::from(typ)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Environment {
    command: String,
    process_args: Vec<String>,
}

impl Environment {
    pub fn get_command(&self) -> String {
        self.command.clone()
    }
    pub fn get_process_args(&self) -> Vec<String> {
        self.process_args.clone()
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct SoftwareFile {
    url: String,
    file_name: String,
    auto_update: bool,
}

impl SoftwareFile {
    pub fn get_url(&self) -> String {
        self.url.clone()
    }
    pub fn get_file_name(&self) -> String {
        self.file_name.clone()
    }
    pub fn is_auto_update(&self) -> bool {
        self.auto_update
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct SystemPlugin {
    local: bool,
    download: String,
    path: String,
}

impl SystemPlugin {
    pub fn is_local(&self) -> bool {
        self.local
    }
    pub fn get_download(&self) -> String {
        self.download.clone()
    }
    pub fn get_file_name(&self) -> String {
        todo!();
    }
    pub fn get_path(&self) -> String {
        self.path.clone()
    }
}

fn get_default_software_config() -> String {
    let json_str = r#"
{
  "software_type": {
    "server": {
      "software_name": [
        {
          "name": "paper",
          "download": "https://api.papermc.io/v2/projects/paper/versions/1.20.4/builds/389/downloads/paper-1.20.4-389.jar",
          "command": "java",
          "max_ram": 1024,
          "ip_path": "server.properties",
          "port_path": "server.properties",
          "system_plugin": {
            "local": false,
            "download": "http://download.codergames.de/minecloud/version/0.1/config/system_plugins/MineCloud-Paper.jar",
            "path": "plugins/"
          },
          "software_lib": [
            "http://download.codergames.de/minecloud/version/0.1/config/software_lib/paper/server.propeties",
            "test2"
          ]
        }
      ]
    },
    "proxy": {
      "software_name": [
        {
          "name": "velocity",
          "download": "https://api.papermc.io/v2/projects/velocity/versions/3.3.0-SNAPSHOT/builds/323/downloads/velocity-3.3.0-SNAPSHOT-323.jar",
          "command": "java",
          "max_ram": 512,
          "ip_path": "velocity.toml",
          "port_path": "velocity.toml",
          "system_plugin": {
            "local": false,
            "download": "http://download.codergames.de/minecloud/version/0.1/config/system_plugins/MineCloud-Velocity.jar",
            "path": "plugins/"
          },
          "software_lib": [
            "http://download.codergames.de/minecloud/version/0.1/config/software_lib/velocity/velocity.toml",
            "test2"
          ]
        }
      ]
    }
  }
}
    "#;
    json_str.to_string()
}
fn default_software_type() -> SoftwareType {
    // Erzeugt eine Standardkonfiguration für `SoftwareType`
    SoftwareType {
        software_name: vec![default_software_name()],
    }
}

fn default_software_name() -> SoftwareName {
    // Erzeugt eine Standardkonfiguration für `SoftwareName`
    SoftwareName {
        name: "paper_default".to_string(),
        software_file: SoftwareFile {
            url: "https://example.com/default_download".to_string(),
            file_name: "default_file.jar".to_string(),
            auto_update: false,
        },
        environment: Environment {
            command: "java".to_string(),
            process_args: vec!["-Xmx%max_ram%M".to_string()],
        },
        max_ram: 1024,
        ip_path: "server.properties".to_string(),
        port_path: "server.properties".to_string(),
        server_type: ServerType::BackendServer,
        system_plugin: SystemPlugin {
            local: true,
            download: "https://example.com/default_plugin".to_string(),
            path: "plugins/".to_string(),
        },
        software_lib: HashMap::new(),
        typ: "default".to_string(),
    }
}
