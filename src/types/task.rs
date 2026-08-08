use rand::RngExt;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::types::installer::Installer;
use crate::types::join_strategy::JoinStrategy;
use crate::types::software_link::SoftwareLink;
use crate::types::template::Template;

/// Represents the configuration and lifecycle rules of a service task.
///
/// A task controls how services are created, scaled, connected to and removed.
/// It defines software, resources, player limits and automatic scaling behaviour.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    /// The unique name of this task.
    ///
    /// Example: `"Lobby"`, `"Survival"`, `"Proxy"`
    name: String,

    /// Character used to split generated service names.
    ///
    /// Example:
    /// `Lobby-1`, `Lobby-2`
    split: char,

    /// List of groups this task belongs to.
    ///
    /// Groups can be used to organize and filter tasks.
    groups: Vec<String>,

    /// Defines whether the service should be deleted after stopping.
    ///
    /// If disabled, stopped services remain available.
    delete_on_stop: bool,

    /// Defines whether this task creates static services.
    ///
    /// Static services are not automatically removed.
    static_service: bool,

    /// Software configuration used by this task.
    ///
    /// Defines the server software type, name and version.
    software: SoftwareLink,

    /// First port used when assigning ports to services.
    start_port: u32,

    /// Maximum RAM allocation for each created service in MB.
    max_ram: u32,

    /// List of allowed nodes where this task can run.
    ///
    /// An empty list allows all nodes.
    nodes: Vec<String>,

    /// Delay before force killing a service during shutdown.
    ///
    /// Value is stored in seconds.
    time_shutdown_before_kill: u64,

    /// Defines whether players can automatically connect to this task.
    ///
    /// Usually enabled for lobby or fallback services.
    default_connect: bool,

    /// Permission required to join this task.
    ///
    /// Empty string means no permission is required.
    join_permission: String,

    /// Maximum amount of players allowed on one service.
    ///
    /// Used for calculating full and empty percentages.
    max_players: u32,

    /// Strategy used when selecting a service for a player connection.
    ///
    /// Examples:
    /// - Fullest
    /// - Emptiest
    /// - RoundRobin
    /// - Random
    join_strategy: JoinStrategy,

    /// Minimum number of services that should always exist.
    ///
    /// This limit is respected regardless of player count.
    min_service_count: u64,

    /// Maximum number of services that may exist.
    ///
    /// A value of `-1` means unlimited services.
    max_service_count: i32,

    /// Percentage at which a service is considered full.
    ///
    /// Example:
    /// `85` means a service with 85% or more players is treated as full.
    full_percent: u32,

    /// Percentage at which a service is considered empty.
    ///
    /// Example:
    /// `5` means a service with 5% or fewer players is considered unused.
    empty_percent: u32,

    /// Minimum number of available (not full) services.
    ///
    /// If fewer services are available, new services may be created.
    min_available_services: u32,

    /// Cooldown time between scaling operations.
    ///
    /// Prevents continuous service creation and removal.
    scale_cooldown_seconds: u32,

    /// Deprecated: Percentage used to detect unused services.
    ///
    /// Use `empty_percent` instead.
    #[deprecated(note = "Use empty_percent instead")]
    percent_of_players_to_check_should_auto_stop_the_service: u32,

    /// Deprecated: Minimum amount of non-full services.
    ///
    /// Use `min_available_services` instead.
    #[deprecated(note = "Use min_available_services instead")]
    min_non_full_service: u32,

    /// Deprecated: Time before unused services are stopped.
    ///
    /// Use the new unused service shutdown configuration instead.
    #[deprecated(note = "Use unused service shutdown configuration instead")]
    auto_stop_time_by_unused_service_in_seconds: u32,

    /// Deprecated: Percentage used to decide when a new service should start.
    ///
    /// Use `full_percent` instead.
    #[deprecated(note = "Use full_percent instead")]
    percent_of_players_for_a_new_service_by_instance: u32,

    /// Installer configuration used when preparing new services.
    installer: Installer,

    /// Templates used when creating new services.
    ///
    /// Templates are copied according to their priority.
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
            time_shutdown_before_kill: 60,
            max_players: 20,
            default_connect: false,
            join_permission: String::new(),
            join_strategy: JoinStrategy::Fullest,
            min_service_count: 0,
            max_service_count: -1,
            full_percent: 85,
            empty_percent: 5,
            min_available_services: 2,
            scale_cooldown_seconds: 30,
            groups: Vec::new(),
            installer: Installer::InstallAll,
            templates: vec![template],

            percent_of_players_to_check_should_auto_stop_the_service: 0,
            min_non_full_service: 0,
            auto_stop_time_by_unused_service_in_seconds: 60,
            percent_of_players_for_a_new_service_by_instance: 0,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_split(&self) -> char {
        self.split
    }
    pub fn set_split(&mut self, split: char) {
        self.split = split;
    }

    pub fn is_delete_on_stop(&self) -> bool {
        self.delete_on_stop
    }
    pub fn set_delete_on_stop(&mut self, value: bool) {
        self.delete_on_stop = value;
    }

    pub fn is_static_service(&self) -> bool {
        self.static_service
    }
    pub fn set_static_service(&mut self, value: bool) {
        self.static_service = value;
    }

    pub fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }
    pub fn set_nodes(&mut self, nodes: Vec<String>) {
        self.nodes = nodes;
    }
    pub fn add_node(&mut self, node: String) {
        self.nodes.push(node);
    }
    pub fn remove_node(&mut self, node: &String) {
        self.nodes.retain(|n| n != node);
    }

    pub fn get_software(&self) -> SoftwareLink {
        self.software.clone()
    }
    pub fn set_software(&mut self, software: SoftwareLink) {
        self.software = software;
    }

    pub fn get_max_ram(&self) -> u32 {
        self.max_ram
    }
    pub fn set_max_ram(&mut self, max_ram: u32) {
        self.max_ram = max_ram;
    }

    pub fn get_start_port(&self) -> u32 {
        self.start_port
    }
    pub fn set_start_port(&mut self, start_port: u32) {
        self.start_port = start_port;
    }

    pub fn get_group_names(&self) -> &Vec<String> {
        &self.groups
    }
    pub fn add_group(&mut self, group: String) {
        self.groups.push(group);
    }
    pub fn remove_group(&mut self, group: &String) {
        self.groups.retain(|g| g != group);
    }
    pub fn clear_groups(&mut self) {
        self.groups.clear();
    }

    pub fn get_join_strategy(&self) -> &JoinStrategy {
        &self.join_strategy
    }

    pub fn set_join_strategy(&mut self, join_strategy: JoinStrategy) {
        self.join_strategy = join_strategy;
    }

    pub fn get_min_service_count(&self) -> u64 {
        self.min_service_count
    }
    pub fn set_min_service_count(&mut self, value: u64) {
        self.min_service_count = value;
    }

    pub fn get_max_service_count(&self) -> i32 {
        self.max_service_count
    }
    pub fn set_max_service_count(&mut self, value: i32) {
        self.max_service_count = value;
    }

    pub fn get_time_shutdown_before_kill(&self) -> Duration {
        Duration::from_secs(self.time_shutdown_before_kill)
    }
    pub fn set_time_shutdown_before_kill(&mut self, secs: u64) {
        self.time_shutdown_before_kill = secs;
    }

    pub fn default_connect(&self) -> bool {
        self.default_connect
    }
    pub fn set_default_connect(&mut self, value: bool) {
        self.default_connect = value;
    }

    pub fn get_join_permission(&self) -> &str {
        &self.join_permission
    }
    pub fn set_join_permission<S: Into<String>>(&mut self, value: S) {
        self.join_permission = value.into();
    }

    pub fn get_max_players(&self) -> u32 {
        self.max_players
    }
    pub fn set_max_players(&mut self, count: u32) {
        self.max_players = count;
    }

    pub fn get_full_percent(&self) -> u32 {
        self.full_percent
    }

    pub fn set_full_percent(&mut self, value: u32) {
        self.full_percent = value.min(100);
    }

    pub fn get_empty_percent(&self) -> u32 {
        self.empty_percent
    }

    pub fn set_empty_percent(&mut self, value: u32) {
        self.empty_percent = value.min(100);
    }

    pub fn get_min_available_services(&self) -> u32 {
        self.min_available_services
    }

    pub fn set_min_available_services(&mut self, value: u32) {
        self.min_available_services = value;
    }

    pub fn get_scale_cooldown_seconds(&self) -> u32 {
        self.scale_cooldown_seconds
    }

    pub fn set_scale_cooldown_seconds(&mut self, value: u32) {
        self.scale_cooldown_seconds = value;
    }

    #[deprecated]
    pub fn get_percent_of_players_to_check_should_auto_stop_the_service(&self) -> u32 {
        self.percent_of_players_to_check_should_auto_stop_the_service
    }

    #[deprecated]
    pub fn set_percent_of_players_to_check_should_auto_stop_the_service(&mut self, value: u32) {
        self.percent_of_players_to_check_should_auto_stop_the_service = value;
    }

    #[deprecated]
    pub fn get_min_non_full_service(&self) -> u32 {
        self.min_non_full_service
    }

    #[deprecated]
    pub fn set_min_non_full_service(&mut self, value: u32) {
        self.min_non_full_service = value;
    }

    #[deprecated]
    pub fn get_auto_stop_time_by_unused_service_in_seconds(&self) -> u32 {
        self.auto_stop_time_by_unused_service_in_seconds
    }

    #[deprecated]
    pub fn set_auto_stop_time_by_unused_service_in_seconds(&mut self, value: u32) {
        self.auto_stop_time_by_unused_service_in_seconds = value;
    }

    #[deprecated]
    pub fn get_percent_of_players_for_a_new_service_by_instance(&self) -> u32 {
        self.percent_of_players_for_a_new_service_by_instance
    }

    #[deprecated]
    pub fn set_percent_of_players_for_a_new_service_by_instance(&mut self, value: u32) {
        self.percent_of_players_for_a_new_service_by_instance = value;
    }

    pub fn get_installer(&self) -> &Installer {
        &self.installer
    }
    pub fn set_installer(&mut self, installer: Installer) {
        self.installer = installer;
    }

    pub fn get_templates(&self) -> Vec<Template> {
        self.templates.clone()
    }
    pub fn add_template(&mut self, template: Template) {
        self.templates.push(template);
    }
    pub fn remove_template(&mut self, template: &Template) {
        self.templates.retain(|t| {
            t.get_prefix() != template.get_prefix() || t.get_name() != template.get_name()
        });
    }
    pub fn clear_templates(&mut self) {
        self.templates.clear();
    }

    pub fn is_delete(&self) -> bool {
        !self.static_service && self.delete_on_stop
    }

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

        self.templates.last()
    }
}

impl TaskRef {
    pub fn new(task: Task) -> Self {
        Self(Arc::new(RwLock::new(task)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, Task> {
        self.0.read().await
    }
    pub async fn write(&self) -> RwLockWriteGuard<'_, Task> {
        self.0.write().await
    }

    pub fn ptr_eq(&self, other: &TaskRef) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub async fn get_name(&self) -> String {
        self.0.read().await.get_name()
    }
}

impl Clone for TaskRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
