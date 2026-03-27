use rand::RngExt;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::types::installer::Installer;
use crate::types::software_link::SoftwareLink;
use crate::types::template::Template;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    name: String,
    split: char,
    groups: Vec<String>,
    delete_on_stop: bool,
    static_service: bool,
    software: SoftwareLink,
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
}

pub struct TaskRef(Arc<RwLock<Task>>);

impl Task {
    pub fn new(name: String, software_link: SoftwareLink, max_ram: u32) -> Task {
        let template = Template::new(&name, "default", 1, false);
        Task {
            name,
            split: '-',
            delete_on_stop: true,
            static_service: false,
            nodes: Vec::new(),
            software: software_link,
            max_ram,
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
            templates: vec![template],
            percent_of_players_for_a_new_service_by_instance: 0,
        }
    }

    pub fn get_name(&self) -> String { self.name.to_string() }
    pub fn set_name(&mut self, name: String) { self.name = name; }

    pub fn get_split(&self) -> char { self.split }
    pub fn set_split(&mut self, split: char) { self.split = split; }

    pub fn is_delete_on_stop(&self) -> bool { self.delete_on_stop }
    pub fn set_delete_on_stop(&mut self, value: bool) { self.delete_on_stop = value; }

    pub fn is_static_service(&self) -> bool { self.static_service }
    pub fn set_static_service(&mut self, value: bool) { self.static_service = value; }

    pub fn get_nodes(&self) -> &Vec<String> { &self.nodes }
    pub fn set_nodes(&mut self, nodes: Vec<String>) { self.nodes = nodes; }
    pub fn add_node(&mut self, node: String) { self.nodes.push(node); }
    pub fn remove_node(&mut self, node: &String) {
        self.nodes.retain(|n| n != node);
    }

    pub fn get_software(&self) -> SoftwareLink { self.software.clone() }
    pub fn set_software(&mut self, software: SoftwareLink) { self.software = software; }

    pub fn get_max_ram(&self) -> u32 { self.max_ram }
    pub fn set_max_ram(&mut self, max_ram: u32) { self.max_ram = max_ram; }

    pub fn get_start_port(&self) -> u32 { self.start_port }
    pub fn set_start_port(&mut self, start_port: u32) { self.start_port = start_port; }

    pub fn get_group_names(&self) -> &Vec<String> { &self.groups }
    pub fn add_group(&mut self, group: String) { self.groups.push(group); }
    pub fn remove_group(&mut self, group: &String) { self.groups.retain(|g| g != group); }
    pub fn clear_groups(&mut self) { self.groups.clear(); }

    pub fn get_min_service_count(&self) -> u64 { self.min_service_count }
    pub fn set_min_service_count(&mut self, value: u64) { self.min_service_count = value; }

    pub fn get_max_service_count(&self) -> i32 { self.max_service_count }
    pub fn set_max_service_count(&mut self, value: i32) { self.max_service_count = value; }

    pub fn get_time_shutdown_before_kill(&self) -> Duration {
        Duration::from_secs(self.time_shutdown_before_kill)
    }
    pub fn set_time_shutdown_before_kill(&mut self, secs: u64) {
        self.time_shutdown_before_kill = secs;
    }

    pub fn default_connect(&self) -> bool { self.default_connect }
    pub fn set_default_connect(&mut self, value: bool) { self.default_connect = value; }

    pub fn get_join_permission(&self) -> &str { &self.join_permission }
    pub fn set_join_permission<S: Into<String>>(&mut self, value: S) {
        self.join_permission = value.into();
    }

    pub fn get_max_players(&self) -> u32 { self.max_players }
    pub fn set_max_players(&mut self, count: u32) { self.max_players = count; }

    pub fn get_percent_of_players_to_check_should_auto_stop_the_service(&self) -> u32 {
        self.percent_of_players_to_check_should_auto_stop_the_service
    }
    pub fn set_percent_of_players_to_check_should_auto_stop_the_service(&mut self, value: u32) {
        self.percent_of_players_to_check_should_auto_stop_the_service = value;
    }

    pub fn get_min_non_full_service(&self) -> u32 { self.min_non_full_service }
    pub fn set_min_non_full_service(&mut self, value: u32) { self.min_non_full_service = value; }

    pub fn get_auto_stop_time_by_unused_service_in_seconds(&self) -> u32 {
        self.auto_stop_time_by_unused_service_in_seconds
    }
    pub fn set_auto_stop_time_by_unused_service_in_seconds(&mut self, value: u32) {
        self.auto_stop_time_by_unused_service_in_seconds = value;
    }

    pub fn get_percent_of_players_for_a_new_service_by_instance(&self) -> u32 {
        self.percent_of_players_for_a_new_service_by_instance
    }
    pub fn set_percent_of_players_for_a_new_service_by_instance(&mut self, value: u32) {
        self.percent_of_players_for_a_new_service_by_instance = value;
    }

    pub fn get_installer(&self) -> &Installer { &self.installer }
    pub fn set_installer(&mut self, installer: Installer) { self.installer = installer; }

    pub fn get_templates(&self) -> Vec<Template> { self.templates.clone() }
    pub fn add_template(&mut self, template: Template) { self.templates.push(template); }
    pub fn remove_template(&mut self, template: &Template) {
        self.templates.retain(|t| {
            t.get_prefix() != template.get_prefix() || t.get_name() != template.get_name()
        });
    }
    pub fn clear_templates(&mut self) { self.templates.clear(); }

    pub fn is_delete(&self) -> bool { !self.static_service && self.delete_on_stop }

    pub fn is_responsible_node(&self, node_name: &str) -> bool {
        self.nodes.is_empty() || self.nodes.iter().any(|n| n == node_name)
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

    pub fn get_template_rng_based_on_priority(&self) -> Option<&Template> {
        if self.templates.is_empty() { return None; }

        let total_weight: u32 = self.templates.iter().map(|t| t.priority).sum();
        if total_weight == 0 { return self.get_template_rng(); }

        let mut rng = rand::rng();
        let mut random_value = rng.random_range(0..total_weight);

        for template in &self.templates {
            if random_value < template.priority {
                return Some(template);
            }
            random_value -= template.priority;
        }

        self.templates.last()
    }
}

impl TaskRef {
    pub fn new(task: Task) -> Self {
        Self(Arc::new(RwLock::new(task)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, Task> { self.0.read().await }
    pub async fn write(&self) -> RwLockWriteGuard<'_, Task> { self.0.write().await }

    pub fn ptr_eq(&self, other: &TaskRef) -> bool { Arc::ptr_eq(&self.0, &other.0) }

    pub async fn get_name(&self) -> String { self.0.read().await.get_name() }
}

impl Clone for TaskRef {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}