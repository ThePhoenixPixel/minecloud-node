use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::CloudConfig;
use crate::types::task::Task;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Template {
    pub template_prefix: String,
    pub name: String,
    pub priority: u32,
    pub always_copy_to_static_services: bool,
}

impl Template {
    pub fn new(
        template_prefix: &str,
        template_name: &str,
        priority: u32,
        always_copy_to_static_services: bool,
    ) -> Template {
        Template {
            template_prefix: template_prefix.to_string(),
            name: template_name.to_string(),
            priority,
            always_copy_to_static_services,
        }
    }

    //template prefix
    pub fn get_prefix(&self) -> &String {
        &self.template_prefix
    }

    pub fn set_prefix(&mut self, template: &String) {
        self.template_prefix = template.clone();
    }

    //name
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: &String) {
        self.name = name.clone();
    }

    //priority
    pub fn get_priority(&self) -> &u32 {
        &self.priority
    }

    pub fn set_priority(&mut self, priority: &u32) {
        self.priority = priority.clone();
    }

    // always_copy_to_static_services
    pub fn is_always_copy_to_static_services(&self) -> bool {
        self.always_copy_to_static_services
    }

    pub fn set_always_copy_to_static_services(&mut self, always_copy_to_static_services: bool) {
        self.always_copy_to_static_services = always_copy_to_static_services;
    }

    pub fn get_path(&self) -> PathBuf {
        PathBuf::from(
            &CloudConfig::get()
                .get_cloud_path()
                .get_template_folder_path(),
        )
        .join(&self.template_prefix)
        .join(&self.name)
    }

    pub fn create_by_task(task: &Task) {
        let mut template_path = CloudConfig::get()
            .get_cloud_path()
            .get_template_folder_path();
        template_path.push(task.get_name());
        template_path.push("default");

        if !template_path.exists() {
            fs::create_dir_all(template_path)
                .expect("Cant create Template Path in 'create_by_task'");
        }
    }

    pub fn create(&self) {
        fs::create_dir_all(&self.get_path()).expect("Cant create Template Path in 'create'");
    }

    pub fn exists(&self) -> bool {
        CloudConfig::get()
            .get_cloud_path()
            .get_template_folder_path()
            .join(&self.get_prefix())
            .join(&self.get_name())
            .is_dir()
    }
}
