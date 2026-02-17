use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::cloud_config::CloudConfig;
use crate::config::software_config::{SoftwareConfig, SoftwareName};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Software {
    pub software_type: String,
    pub name: String,
}

impl Software {
    pub fn new(software_name: &SoftwareName) -> Software {
        Software {
            software_type: software_name.get_typ(),
            name: software_name.get_name(),
        }
    }

    pub fn get_software_name(&self) -> SoftwareName {
        SoftwareConfig::get_software(&self.get_software_type(), &self.get_name())
    }

    //software type
    pub fn get_software_type(&self) -> String {
        self.software_type.clone()
    }

    pub fn set_software_type(&mut self, software_type: &String) {
        self.software_type = software_type.clone();
    }

    //name
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_server_file_name(&self) -> String {
        self.get_software_name().get_software_file().get_file_name()
    }

    pub fn set_name(&mut self, name: &String) {
        self.name = name.clone();
    }

    pub fn get_software_file_path(&self) -> PathBuf {
        let mut software_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_software_files_folder_path();
        software_path.push(&self.get_software_type());
        software_path.push(self.get_server_file_name());
        software_path
    }

    pub fn get_system_plugin_path(&self) -> PathBuf {
        let mut system_plugin_path = CloudConfig::get()
            .get_cloud_path()
            .get_system_folder()
            .get_system_plugins_folder_path();

        system_plugin_path.push(&self.get_software_type());
        system_plugin_path.push(format!("MineCloud-{}", &self.get_server_file_name()));
        system_plugin_path
    }

    pub fn get_system_plugin_name(&self) -> String {
        // hier muss der name des plugin noch rein derzeit wird einfach name.ext der software verwendet
        format!("MineCloud-{}", &self.get_server_file_name())
    }
}
