use serde::{Deserialize, Serialize};
use crate::types::{Installer, Software, Task, Template};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServiceConfig {
    software: Software,
    max_ram: u32,
    max_players: u32,
    start_port: u32,
    templates: Vec<Template>,
    installer: Installer,
}

impl ServiceConfig {
    pub fn get_software(&self) -> &Software {
        &self.software
    }

    pub fn get_max_ram(&self) -> u32 {
        self.max_ram
    }

    pub fn get_max_players(&self) -> u32 {
        self.max_players
    }

    pub fn get_start_port(&self) -> u32 {
        self.start_port
    }

    pub fn get_templates(&self) -> &[Template] {
        &self.templates
    }

    pub fn get_installer(&self) -> &Installer {
        &self.installer
    }
}

impl From<&Task> for ServiceConfig {
    fn from(task: &Task) -> Self {
        Self {
            software: task.get_software().clone(),
            max_ram: task.get_max_ram(),
            max_players: task.get_max_players(),
            start_port: task.get_start_port(),
            templates: task.get_templates().to_vec(),
            installer: task.get_installer().clone(),
        }
    }
}

