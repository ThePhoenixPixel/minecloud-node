use bx::network::address::Address;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::{CloudConfig, SoftwareName};
use crate::types::task::Task;
use crate::types::{EntityId, ServiceConfig, ServiceStatus};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    id: EntityId,
    name: String,
    status: ServiceStatus,
    parent_node: String,
    current_players: u32,
    started_at: Option<NaiveDateTime>,
    stopped_at: Option<NaiveDateTime>,
    idle_since: Option<NaiveDateTime>,
    server_listener: Address,
    plugin_listener: Address,
    cloud_listener: Address,
    #[deprecated]
    task: Task,
    config: ServiceConfig,
    task_name: String,
}

impl Service {
    pub(crate) fn new(id: EntityId, name: String, task: &Task, config: &Arc<CloudConfig>) -> Service {
        Service {
            id,
            name,
            status: ServiceStatus::Stopped,
            parent_node: config.get_name(),
            current_players: 0,
            started_at: None,
            stopped_at: None,
            idle_since: None,
            server_listener: Address::new(&config.get_server_host(), &0),
            plugin_listener: Address::get_local_ipv4(),
            cloud_listener: config.get_node_host(),
            task_name: task.get_name(),
            config: ServiceConfig::from(task),
            task: task.clone(),
        }
    }

    pub fn get_id(&self) -> &EntityId {
        &self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_status(&self) -> ServiceStatus {
        self.status
    }

    pub fn set_status(&mut self, status: ServiceStatus) {
        self.status = status;
    }

    pub fn get_parent_node(&self) -> &str {
        &self.parent_node
    }

    pub fn get_current_players(&self) -> u32 {
        self.current_players
    }

    pub fn set_current_player(&mut self, count: u32) {
        self.current_players = count;
    }

    pub fn get_started_at(&self) -> Option<NaiveDateTime> {
        self.started_at
    }

    pub fn get_stopped_at(&self) -> Option<NaiveDateTime> {
        self.stopped_at
    }

    pub fn get_idle_since(&self) -> Option<NaiveDateTime> {
        self.idle_since
    }

    pub fn start_idle_timer(&mut self) {
        self.idle_since = Some(Utc::now().naive_utc());
    }

    pub fn get_server_listener(&self) -> &Address {
        &self.server_listener
    }

    pub fn set_server_listener(&mut self, address: Address) {
        self.server_listener = address
    }
    pub fn get_plugin_listener(&self) -> &Address {
        &self.plugin_listener
    }

    pub fn set_plugin_listener(&mut self, address: Address) {
        self.plugin_listener = address;
    }

    pub fn get_cloud_listener(&self) -> &Address {
        &self.cloud_listener
    }

    pub fn set_cloud_listener(&mut self, address: Address) {
        self.cloud_listener = address;
    }

    #[deprecated]
    pub fn get_task(&self) -> &Task {
        &self.task
    }

    pub fn get_config(&self) -> &ServiceConfig {
        &self.config
    }

    pub fn is_start(&self) -> bool {
        self.status == ServiceStatus::Starting || self.status == ServiceStatus::Running
    }
    pub fn is_stop(&self) -> bool {
        self.status == ServiceStatus::Stopped
            || self.status == ServiceStatus::Stopping
            || self.status == ServiceStatus::Failed
    }

    pub fn is_failed(&self) -> bool {
        self.status == ServiceStatus::Failed
    }

    #[deprecated]
    pub fn is_proxy(&self) -> bool {
        self.get_software_name().get_server_type().is_proxy()
    }

    #[deprecated]
    pub fn is_backend_server(&self) -> bool {
        self.get_software_name()
            .get_server_type()
            .is_backend_server()
    }

    #[deprecated]
    pub fn is_local(&self) -> bool {
        self.parent_node == CloudConfig::get().get_name()
    }

    #[deprecated]
    pub fn get_started_at_to_string(&self) -> Option<String> {
        if let Some(date) = self.started_at {
            return Some(date.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        None
    }

    #[deprecated]
    pub fn get_stopped_at_to_string(&self) -> Option<String> {
        if let Some(date) = self.stopped_at {
            return Some(date.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        None
    }

    #[deprecated]
    pub fn get_idle_since_to_string(&self) -> Option<String> {
        if let Some(date) = self.stopped_at {
            return Some(date.format("%Y-%m-%d %H:%M:%S").to_string());
        }
        None
    }

    #[deprecated]
    pub fn get_software_name(&self) -> SoftwareName {
        self.get_task().get_software().get_software_name()
    }
}
