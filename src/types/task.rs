use bx::path::Directory;
use rand::RngExt;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{fs, io};
use std::sync::Arc;
use std::time::Duration;

use crate::types::group::Group;
use crate::types::installer::Installer;
use crate::types::software::Software;
use crate::types::template::Template;
use crate::config::{CloudConfig, SoftwareConfig};
use crate::{error, log_error, log_info};
use crate::utils::error::*;

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
    min_service_count: u64,
    max_service_count: i32,
    time_shutdown_before_kill: u64,
    default_connect: bool,
    join_permission: String,
    max_players: u32,
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
            max_service_count: -1,
            time_shutdown_before_kill: 5000,
            default_connect: false,
            join_permission: String::new(),
            max_players: 20,
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

    pub fn update(&mut self, new_task: Task) {
        self.delete_as_file();

        self.name = new_task.name;
        self.split = new_task.split;
        self.groups = new_task.groups;
        self.delete_on_stop = new_task.delete_on_stop;
        self.static_service = new_task.static_service;
        self.software = new_task.software;
        self.start_port = new_task.start_port;
        self.max_ram = new_task.max_ram;
        self.nodes = new_task.nodes;
        self.min_service_count = new_task.min_service_count;
        self.max_service_count = new_task.max_service_count;
        self.default_connect = new_task.default_connect;
        self.join_permission = new_task.join_permission;
        self.percent_of_players_to_check_should_auto_stop_the_service =
            new_task.percent_of_players_to_check_should_auto_stop_the_service;
        self.min_non_full_service = new_task.min_non_full_service;
        self.auto_stop_time_by_unused_service_in_seconds =
            new_task.auto_stop_time_by_unused_service_in_seconds;
        self.percent_of_players_for_a_new_service_by_instance =
            new_task.percent_of_players_for_a_new_service_by_instance;
        self.installer = new_task.installer;
        self.templates = new_task.templates;

        self.save_to_file();
    }

    // Getter and Setter for name
    pub fn get_name(&self) -> String {
        self.name.to_string()
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
    pub fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
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
    pub fn get_groups(&self) -> Vec<Group> {
        let mut groups: Vec<Group> = Vec::new();
        for group_str in self.groups.clone() {
            if let Some(group) = Group::get_from_name(&group_str) {
                groups.push(group);
            }
        }
        groups
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
    pub fn get_min_service_count(&self) -> u64 {
        self.min_service_count
    }

    pub fn set_min_service_count(&mut self, min_service_count: u64) {
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
    
    pub fn get_time_shutdown_before_kill(&self) -> Duration {
        Duration::from_secs(self.time_shutdown_before_kill)
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

    // max players
    pub fn get_max_players(&self) -> u32 {
        self.max_players
    }

    pub fn set_max_players(&mut self, count: u32) {
        self.max_players = count;
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

    pub fn is_startup_local(&self, config: &Arc<CloudConfig>) -> bool {
        let nodes = self.get_nodes();
        nodes.is_empty() || nodes.iter().any(|n| *n == config.get_name())
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

    pub fn get_templates_sorted_by_priority(&self) -> Vec<Template> {
        let mut templates = self.get_templates();
        templates.sort_by(|a, b| a.priority.cmp(&b.priority));
        templates
    }

    pub fn get_templates_sorted_by_priority_desc(&self) -> Vec<Template> {
        let mut templates = self.get_templates();
        templates.sort_by(|a, b| b.priority.cmp(&a.priority));
        templates
    }

    pub fn get_template_rng(&self) -> Option<&Template> {
        let mut rng = rand::rng();
        self.templates.choose(&mut rng)
    }

    // Select Template based on Priority (higher priority = higher chance)
    pub fn get_template_rng_based_on_priority(&self) -> Option<&Template> {
        if self.templates.is_empty() {
            return None;
        }

        let total_weight: u32 = self.templates.iter().map(|t| t.priority).sum();

        if total_weight == 0 {
            return self.get_template_rng();
        }

        let mut rng = rand::rng();
        let mut random_value = rng.random_range(0..total_weight);

        for template in &self.templates {
            if random_value < template.priority {
                return Some(template);
            }
            random_value -= template.priority;
        }

        // fallback
        self.templates.last()
    }

    pub fn prepared_to_service(&self) -> Result<PathBuf, CloudError> {
        // create the next free service folder with the template
        let target_path = self.create_next_free_service_folder()?;
        for group in self.get_groups() {
            group.install_in_path(&target_path)?;
        }

        let mut templates: Vec<Template> = Vec::new();
        match self.get_installer() {
            Installer::InstallAll => templates = self.get_templates_sorted_by_priority(),
            Installer::InstallAllDesc => templates = self.get_templates_sorted_by_priority_desc(),
            Installer::InstallRandom => match self.get_template_rng() {
                Some(template) => templates.push(template.clone()),
                None => return Err(error!(TemplateNotFound)),
            },
            Installer::InstallRandomWithPriority => {
                match self.get_template_rng_based_on_priority() {
                    Some(template) => templates.push(template.clone()),
                    None => return Err(error!(TemplateNotFound)),
                }
            }
        }

        for template in templates {
            Directory::copy_folder_contents(&template.get_path(), &target_path)
                .map_err(|e| error!(CantCopyTemplateToNewServiceFolder, e))?;
        }
        Ok(target_path)
    }

    // create the next not exist service folder
    fn create_next_free_service_folder(&self) -> Result<PathBuf, CloudError> {
        let mut folder_index: u32 = 1;
        let target_base_path = self.get_service_path();
        let mut target_service_folder_path =
            target_base_path.join(format!("{}{}{}", self.get_name(), self.get_split(), folder_index));

        while target_service_folder_path.exists() {
            folder_index += 1;
            target_service_folder_path =
                target_base_path.join(format!("{}{}{}", self.get_name(), self.get_split(), folder_index));
        }
        fs::create_dir_all(&target_service_folder_path)
            .map_err(|e| error!(CantCreateServiceFolder, e))?;
        Ok(target_service_folder_path)
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
        log_info!(
            "groups: {:?}",
            self.get_groups().iter().map(|g| g.get_name())
        );
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
