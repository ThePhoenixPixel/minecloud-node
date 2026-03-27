use bx::network::address::Address;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::CloudConfig;
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
    config: ServiceConfig,
    task_name: String,
    default_connect: bool,
    join_permission: String,
}

impl Service {
    pub fn new(id: EntityId, name: String, task: &Task, config: &Arc<CloudConfig>) -> Service {
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
            default_connect: task.default_connect(),
            join_permission: task.get_join_permission().to_string(),
            config: ServiceConfig::from(task),
        }
    }

    pub fn get_id(&self) -> &EntityId { &self.id }
    pub fn get_name(&self) -> &str { &self.name }

    pub fn get_status(&self) -> ServiceStatus { self.status }
    pub fn set_status(&mut self, status: ServiceStatus) { self.status = status; }

    pub fn get_parent_node(&self) -> &str { &self.parent_node }
    pub fn is_local_node(&self, node_name: &str) -> bool { self.parent_node == node_name }

    pub fn get_current_players(&self) -> u32 { self.current_players }
    pub fn set_current_player(&mut self, count: u32) { self.current_players = count; }

    pub fn get_started_at(&self) -> Option<NaiveDateTime> { self.started_at }
    pub fn get_stopped_at(&self) -> Option<NaiveDateTime> { self.stopped_at }
    pub fn get_idle_since(&self) -> Option<NaiveDateTime> { self.idle_since }

    pub fn start_idle_timer(&mut self) { self.idle_since = Some(Utc::now().naive_utc()); }

    pub fn get_server_listener(&self) -> &Address { &self.server_listener }
    pub fn set_server_listener(&mut self, address: Address) { self.server_listener = address; }

    pub fn get_plugin_listener(&self) -> &Address { &self.plugin_listener }
    pub fn set_plugin_listener(&mut self, address: Address) { self.plugin_listener = address; }

    pub fn get_cloud_listener(&self) -> &Address { &self.cloud_listener }
    pub fn set_cloud_listener(&mut self, address: Address) { self.cloud_listener = address; }

    pub fn get_task_name(&self) -> &str { &self.task_name }

    pub fn default_connect(&self) -> bool { self.default_connect }

    pub fn get_join_permission(&self) -> &str { &self.join_permission }

    pub fn get_config(&self) -> &ServiceConfig { &self.config }

    pub fn is_proxy(&self) -> bool {
        self.config.get_software().get_software_type().is_proxy()
    }

    pub fn is_backend_server(&self) -> bool {
        self.config.get_software().get_software_type().is_backend_server()
    }

    pub fn is_start(&self) -> bool {
        self.status == ServiceStatus::Starting || self.status == ServiceStatus::Running
    }

    pub fn is_running(&self) -> bool {
        self.status == ServiceStatus::Running
    }

    pub fn is_stop(&self) -> bool {
        matches!(self.status, ServiceStatus::Stopped | ServiceStatus::Stopping | ServiceStatus::Failed)
    }

    pub fn is_failed(&self) -> bool { self.status == ServiceStatus::Failed }
}