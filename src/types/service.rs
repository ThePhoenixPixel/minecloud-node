use bx::network::address::Address;
use bx::path::Directory;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::{CloudConfig, SoftwareName};
use crate::types::task::Task;
use crate::types::{EntityId, ServiceStatus};
use crate::utils::error::*;
use crate::{error, log_info};

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
    task: Task,
}

impl Service {
    pub(crate) fn new(id: EntityId, name: String, task: Task, config: Arc<CloudConfig>) -> Service {
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
            task,
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

    pub fn get_task(&self) -> &Task {
        &self.task
    }

    pub fn is_delete(&self) -> bool {
        !self.task.is_static_service() && self.task.is_delete_on_stop()
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

    // Todo: old

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

    #[deprecated]
    pub fn get_path(&self) -> PathBuf {
        self.get_task().get_service_path().join(self.get_name())
    }

    #[deprecated]
    pub fn get_path_with_server_file(&self) -> PathBuf {
        self.get_path()
            .join(self.get_task().get_software().get_server_file_name())
    }

    #[deprecated]
    pub fn get_path_with_service_config(&self) -> PathBuf {
        self.get_path().join(".minecloud")
    }

    #[deprecated]
    pub fn get_path_with_service_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("service_config.json")
    }

    #[deprecated]
    pub fn get_path_stdout_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stdout.log")
    }
    #[deprecated]

    pub fn get_path_stdin_file(&self) -> PathBuf {
        self.get_path_with_service_config().join("server_stdin.log")
    }

    #[deprecated]
    pub fn get_path_stderr_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stderr.log")
    }

    #[deprecated]
    // wie viele services muss ich noch starten???
    pub fn get_starts_service_from_task(task: &Task) -> u64 {
        let service_path = task.get_service_path();
        let mut start_service: u64 = 0;
        let files_name = Directory::get_files_name_from_path(&service_path);

        for file_name in files_name {
            let mut current_service_path = service_path.clone();
            if file_name.starts_with(&task.get_name()) {
                current_service_path.push(file_name);

                if Service::is_service_start_or_prepare(&mut current_service_path) {
                    start_service += 1;
                }
            }
        }
        start_service
    }

    #[deprecated]
    pub fn is_service_start_or_prepare(path: &mut PathBuf) -> bool {
        match get_from_path(path) {
            Some(service) => service.is_start() || !service.is_stop(),
            None => false,
        }
    }

    #[deprecated]
    pub fn get_from_name(name: &String) -> Option<Service> {
        let mut path = CloudConfig::get()
            .get_cloud_path()
            .get_service_folder()
            .get_temp_folder_path()
            .join(&name);
        get_from_path(&mut path)
    }

    #[deprecated]
    pub fn install_software(&self) -> Result<(), CloudError> {
        let target_path = self
            .get_path()
            .join(&self.get_task().get_software().get_server_file_name());
        let software_path = self.get_task().get_software().get_software_file_path();

        fs::copy(&software_path, &target_path).map_err(|e| error!(CantCopySoftware, e))?;
        Ok(())
    }

    #[deprecated]
    pub fn install_system_plugin(&self) -> Result<(), CloudError> {
        let software = self.get_software_name();
        let system_plugin_path = self.get_task().get_software().get_system_plugin_path();
        let mut target_path = self
            .get_path()
            .join(&software.get_system_plugin().get_path());

        if !target_path.exists() {
            fs::create_dir_all(&target_path).map_err(|e| error!(CantCreateSystemPluginPath, e))?;
        }

        target_path.push(self.get_task().get_software().get_system_plugin_name());

        if !system_plugin_path.exists() {
            return Err(error!(CantFindSystemPlugin));
        }

        match fs::copy(system_plugin_path, target_path) {
            Ok(_) => {
                log_info!("Successfully install the System Plugin");
                Ok(())
            }
            Err(e) => Err(error!(CantCopySystemPlugin, e)),
        }
    }

    #[deprecated]
    pub fn install_software_lib(&self, config: &CloudConfig) -> Result<(), CloudError> {
        let software_lib_path = config
            .get_cloud_path()
            .get_system_folder()
            .get_software_lib_folder_path()
            .join(self.get_task().get_software().get_software_type())
            .join(self.get_task().get_software().get_name());

        Directory::copy_folder_contents(&software_lib_path, &self.get_path())
            .map_err(|e| error!(Internal, e))
    }
}
#[deprecated]
fn get_from_path(path: &mut PathBuf) -> Option<Service> {
    //path -> /service/temp/Lobby-1/
    path.push(".minecloud");
    path.push("service_config.json");
    if let Ok(file_content) = read_to_string(path) {
        if let Ok(service) = serde_json::from_str(&file_content) {
            Some(service)
        } else {
            None
        }
    } else {
        None
    }
}
