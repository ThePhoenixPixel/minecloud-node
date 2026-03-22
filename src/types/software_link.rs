use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::{CloudConfig, SoftwareConfig, Software};
use crate::types::SoftwareType;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[derive(Eq, Hash, PartialEq)]
pub struct SoftwareLink {
    typ: SoftwareType,
    name: String,
    version: String,
}

impl SoftwareLink {

    pub fn new_from_row(typ: SoftwareType, name: String, version: String) -> SoftwareLink {
        SoftwareLink {
            typ,
            name,
            version,
        }
    }

    pub fn new(software: &Software) -> SoftwareLink {
        SoftwareLink {
            typ: software.get_typ().clone(),
            name: software.get_name().to_string(),
            version: software.get_version().to_string(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_software_type(&self) -> &SoftwareType {
        &self.typ
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }



   /* pub fn get_software_name(&self) -> Software {
        SoftwareConfig::find_software(&self.get_software_type(), &self.get_name()).unwrap()
    }

    */

    //name
    pub fn get_server_file_name(&self) -> String {
        todo!("get server file name");
        //self.get_software_name().get_software_file().get_file_name()
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
