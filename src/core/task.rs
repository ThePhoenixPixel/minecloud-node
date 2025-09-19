use bx::path::Directory;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, io};

use crate::core::installer::Installer;
use crate::core::software::Software;
use crate::core::template::Template;
use crate::sys_config::cloud_config::CloudConfig;
use crate::sys_config::software_config::SoftwareConfig;
use crate::utils::logger::Logger;
use crate::{log_error, log_info};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    name: String,
    split: char,
    groups: Vec<String>,
    delete_on_stop: bool,
    static_service: bool,
    software: Software,
    start_port: u32,
    max_ram: u32,
    nodes: Vec<String>,
    min_service_count: u32,
    max_service_count: i32,
    default_connect: bool,
    join_permission: String,
    percent_of_players_to_check_should_auto_stop_the_service: u32,
    min_non_full_service: u32,
    auto_stop_time_by_unused_service_in_seconds: u32,
    percent_of_players_for_a_new_service_by_instance: u32,
    installer: Installer,
    templates: Vec<Template>,
    // includes: Include,
    // delete_files_after_stop: Vec<String>,
    // --copy_files_after_stop:
    //
}

impl Task {
    pub fn create(
        name: &String,
        software_type: &String,
        software_name: &String,
    ) -> Result<Task, &'static str> {
        // check if Task exists
        if Task::is_exist(name.clone()) {
            return Err("Task Esistierts bereits");
        }

        // check if Software exists
        if !SoftwareConfig::is_exists(software_type, software_name) {
            return Err("Software NOT Found");
        }

        let software = SoftwareConfig::get_software(software_type, software_name);
        let template = Template::new(&name, "default", 1, false);
        let task = Task {
            name: name.to_string(),
            split: '-',
            delete_on_stop: true,
            static_service: false,
            nodes: Vec::new(),
            software: Software::new(&software),
            max_ram: software.get_max_ram(),
            start_port: 40000,
            min_service_count: 0,
            //prepared_service_count: 0,
            max_service_count: -1,
            default_connect: false,
            join_permission: String::new(),
            percent_of_players_to_check_should_auto_stop_the_service: 0,
            min_non_full_service: 0,
            auto_stop_time_by_unused_service_in_seconds: 60,
            groups: Vec::new(),
            installer: Installer::InstallAll,
            templates: vec![template.clone()],
            percent_of_players_for_a_new_service_by_instance: 0,
        };

        template.create();
        task.save_to_file();
        Ok(task)
    }

    // Getter and Setter for name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn change_name(&mut self, name: String) {
        self.delete_as_file();
        self.name = name;
        self.save_to_file();
    }

    //getter und setter for split
    pub fn get_split(&self) -> char {
        self.split
    }

    pub fn set_split(&mut self, split: &char) {
        self.split = split.clone();
        self.save_to_file()
    }

    // Getter and Setter for delete_on_stop
    pub fn is_delete_on_stop(&self) -> bool {
        self.delete_on_stop
    }

    pub fn set_delete_on_stop(&mut self, delete_on_stop: bool) {
        self.delete_on_stop = delete_on_stop;
        self.save_to_file();
    }

    // Getter and Setter for static_service
    pub fn is_static_service(&self) -> bool {
        self.static_service
    }

    pub fn set_static_service(&mut self, static_service: bool) {
        self.static_service = static_service;
        self.save_to_file();
    }

    // Getter and Setter for nodes
    pub fn get_nodes(&self) -> Vec<String> {
        self.nodes.clone()
    }

    pub fn add_node(&mut self, node: String) {
        self.nodes.push(node);
        self.save_to_file();
    }

    pub fn remove_node(&mut self, node: &String) {
        if let Some(index) = self.nodes.iter().position(|n| n == node) {
            self.nodes.remove(index);
        }
        self.save_to_file();
    }

    pub fn clear_nodes(&mut self) {
        self.nodes.clear();
        self.save_to_file();
    }

    pub fn is_startup_local(&self) -> bool {
        if !self.get_nodes().is_empty() {
            for node in self.get_nodes() {
                return node == CloudConfig::get().get_name();
            }
            false
        } else {
            true
        }
    }

    // Getter and Setter for software
    pub fn get_software(&self) -> Software {
        self.software.clone()
    }

    pub fn set_software(&mut self, software: Software) {
        self.software = software;
        self.save_to_file();
    }

    //max ram
    pub fn get_max_ram(&self) -> u32 {
        self.max_ram
    }

    pub fn set_max_ram(&mut self, max_ram: &u32) {
        self.max_ram = max_ram.clone();
        self.save_to_file();
    }

    // Getter and Setter for start_port
    pub fn get_start_port(&self) -> u32 {
        self.start_port
    }

    pub fn set_start_port(&mut self, start_port: u32) {
        self.start_port = start_port;
        self.save_to_file();
    }

    // Getter and Setter for groups
    pub fn get_groups(&self) -> &Vec<String> {
        &self.groups
    }

    pub fn add_group(&mut self, group: &String) {
        self.groups.push(group.clone());
        self.save_to_file();
    }

    pub fn remove_group(&mut self, group: &String) {
        if let Some(index) = self.groups.iter().position(|g| g == group) {
            self.groups.remove(index);
        }
        self.save_to_file();
    }

    pub fn clear_groups(&mut self) {
        self.groups.clear();
        self.save_to_file();
    }

    // Getter and Setter for min_service_count
    pub fn get_min_service_count(&self) -> u32 {
        self.min_service_count
    }

    pub fn set_min_service_count(&mut self, min_service_count: u32) {
        self.min_service_count = min_service_count;
        self.save_to_file();
    }

    // max_service_count
    pub fn get_max_service_count(&self) -> i32 {
        self.max_service_count
    }

    pub fn set_max_service_count(&mut self, max_service_count: i32) {
        self.max_service_count = max_service_count;
    }

    // default_connect
    pub fn default_connect(&self) -> bool {
        self.default_connect
    }

    pub fn set_default_connect(&mut self, value: bool) {
        self.default_connect = value;
    }

    // join_permission
    pub fn get_join_permission(&self) -> &str {
        &self.join_permission
    }

    pub fn set_join_permission<S: Into<String>>(&mut self, value: S) {
        self.join_permission = value.into();
    }

    // percent_of_players_to_check_should_auto_stop_the_service
    pub fn get_percent_of_players_to_check_should_auto_stop_the_service(&self) -> u32 {
        self.percent_of_players_to_check_should_auto_stop_the_service
    }

    pub fn set_percent_of_players_to_check_should_auto_stop_the_service(&mut self, value: u32) {
        self.percent_of_players_to_check_should_auto_stop_the_service = value;
    }

    // min_non_full_service
    pub fn get_min_non_full_service(&self) -> u32 {
        self.min_non_full_service
    }

    pub fn set_min_non_full_service(&mut self, value: u32) {
        self.min_non_full_service = value;
    }

    // auto_stop_time_by_unused_service_in_seconds
    pub fn get_auto_stop_time_by_unused_service_in_seconds(&self) -> u32 {
        self.auto_stop_time_by_unused_service_in_seconds
    }

    pub fn set_auto_stop_time_by_unused_service_in_seconds(&mut self, value: u32) {
        self.auto_stop_time_by_unused_service_in_seconds = value;
    }

    // percent_of_players_for_a_new_service_by_instance
    pub fn get_percent_of_players_for_a_new_service_by_instance(&self) -> u32 {
        self.percent_of_players_for_a_new_service_by_instance
    }

    pub fn set_percent_of_players_for_a_new_service_by_instance(&mut self, value: u32) {
        self.percent_of_players_for_a_new_service_by_instance = value;
    }

    // Installer
    pub fn get_installer(&self) -> &Installer {
        &self.installer
    }

    pub fn set_installer(&mut self, installer: &Installer) {
        self.installer = installer.clone();
        self.save_to_file();
    }

    // Template
    pub fn get_templates(&self) -> Vec<Template> {
        self.templates.clone()
    }

    pub fn add_template(&mut self, template: Template) {
        self.templates.push(template);
        self.save_to_file();
    }

    pub fn remove_template(&mut self, template: Template) {
        if let Some(index) = self.templates.iter().position(|task_template| {
            task_template.get_prefix() == template.get_prefix()
                && task_template.get_name() == template.get_name()
        }) {
            self.templates.remove(index);
            self.save_to_file();
        }
    }

    pub fn clear_templates(&mut self) {
        self.templates.clear();
        self.save_to_file();
    }

    pub fn is_exist(name: String) -> bool {
        if Task::get_task(&name).is_some() {
            true
        } else {
            false
        }
    }

    // get task object from name
    pub fn get_task(name: &str) -> Option<Task> {
        let task_path = CloudConfig::get().get_cloud_path().get_task_folder_path();

        let files_name = Directory::get_files_name_from_path(&task_path);

        // iter list of files Name
        for file_name in files_name {
            let task = match Task::from_path(&task_path.join(&file_name)) {
                Ok(task) => task,
                Err(e) => {
                    log_error!("{}", e.to_string());
                    return None;
                }
            };

            // check name of the task is the same of the param name
            if task.get_name() == name {
                return Some(task);
            }
        }
        None
    }

    // from path to task object
    pub fn from_path(path: &PathBuf) -> io::Result<Task> {
        let mut file = File::open(path)?;
        let mut content = String::new();

        file.read_to_string(&mut content)?;

        let task: Task = serde_json::from_str(&content)?;

        Ok(task)
    }

    pub fn get_task_all() -> Vec<Task> {
        let task_path = CloudConfig::get().get_cloud_path().get_task_folder_path();
        let mut tasks: Vec<Task> = Vec::new();

        if task_path.exists() && task_path.is_dir() {
            if let Ok(entries) = fs::read_dir(task_path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if let Some(name) = file_name.strip_suffix(".json") {
                                tasks.push(match Task::get_task(&name.to_string()) {
                                    Some(task) => task,
                                    None => break,
                                });
                            }
                        }
                    }
                }
            }
        }

        tasks
    }

    pub fn get_all() -> Vec<String> {
        let mut tasks: Vec<String> = Vec::new();
        for task in Task::get_task_all() {
            tasks.push(task.get_name().to_string());
        }
        tasks
    }

    pub fn save_to_file(&self) {
        let serialized_task =
            serde_json::to_string_pretty(&self).expect("Error beim Serialisieren der Task");
        let task_path = CloudConfig::get()
            .get_cloud_path()
            .get_task_folder_path()
            .join(format!("{}.json", self.get_name()));

        if !task_path.exists() {
            Template::create_by_task(&self);
        }

        let mut file = File::create(&task_path).expect("Error beim Erstellen der Task-Datei");
        file.write_all(serialized_task.as_bytes())
            .expect("Error beim Schreiben in die Task-Datei");
    }

    pub fn delete_as_file(&self) {
        let mut task_path = CloudConfig::get().get_cloud_path().get_task_folder_path();
        task_path.push(format!("{}.json", &self.name));

        fs::remove_file(task_path).expect("Error bei  removen der task datei");
    }

    pub fn prepared_to_service(&self) -> Result<PathBuf, String> {
        // create the next free service folder with the template
        let target_path = &self.create_next_free_service_folder();
        let templates = &self.get_templates();
        let template = match select_template_with_priority(&templates) {
            Some(template) => template,
            None => {
                return Err(
                    format!("Kein Template gefunden für Task {}", &self.get_name()).to_string(),
                );
            }
        };

        // copy the template in the new service folder
        match Directory::copy_folder_contents(&template.get_path(), &target_path) {
            Ok(_) => Ok(target_path.clone()),
            Err(e) => Err(format!("Error beim Copy the Template \n {}", e.to_string())),
        }
    }

    // create the next not exist service folder
    fn create_next_free_service_folder(&self) -> PathBuf {
        let mut folder_index: u32 = 1;
        let target_base_path = self.get_service_path();
        let mut target_service_folder_path =
            target_base_path.join(format!("{}-{}", &self.get_name(), folder_index));

        while target_service_folder_path.exists() {
            folder_index += 1;
            target_service_folder_path =
                target_base_path.join(format!("{}-{}", &self.get_name(), folder_index));
        }

        Directory::create_path(&target_service_folder_path);
        target_service_folder_path
    }

    //get temp or static for the service
    pub fn get_service_path(&self) -> PathBuf {
        let path = if self.static_service {
            CloudConfig::get()
                .get_cloud_path()
                .get_service_folder()
                .get_static_folder_path()
        } else {
            CloudConfig::get()
                .get_cloud_path()
                .get_service_folder()
                .get_temp_folder_path()
        };
        path
    }

    //print the task object in cmd
    pub fn print(&self) {
        log_info!("--------> Task Info <--------");
        log_info!("name: {}", self.get_name());
        log_info!("split: {}", self.get_split());
        log_info!("delete_on_stop: {}", self.is_delete_on_stop());
        log_info!("static_service: {}", self.is_static_service());
        log_info!("nodes: {:?}", self.get_nodes());
        log_info!("software: ");
        log_info!(
            "     software_type: {}",
            self.get_software().get_software_type()
        );
        log_info!("     name: {}", self.get_software().get_name());
        log_info!("max_ram: {}", self.get_max_ram());
        log_info!("start_port: {}", self.get_start_port());
        log_info!("min_service_count: {}", self.get_min_service_count());
        log_info!("groups: {:?}", self.get_groups());
        log_info!("installer: {:?}", self.get_installer());
        log_info!("templates: ");
        for template in self.get_templates() {
            log_info!("     prefix: {}", template.get_prefix());
            log_info!("     name: {}", template.get_name());
            log_info!("     priority: {}", template.get_priority());
        }
        log_info!("-----------------------------");
    }
}

fn select_template_with_priority(templates: &[Template]) -> Option<&Template> {
    let mut rng = rand::rng();
    let total_priority: u32 = templates.iter().map(|t| t.priority).sum();
    let mut rand_value = rng.random_range(1..=total_priority);

    for template in templates {
        if rand_value <= template.priority {
            return Some(template);
        }
        rand_value -= template.priority;
    }
    None
}
