use bx::network::address::Address;
use bx::network::url::{Url, UrlSchema};
use bx::path::Directory;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;

use crate::types::task::Task;
use crate::config::cloud_config::CloudConfig;
use crate::config::software_config::SoftwareName;
use crate::utils::error::*;
use crate::{error, log_error, log_info, log_warning};
use crate::manager::service_manager::ServiceManager;
use crate::types::{EntityId, ServiceStatus};
use crate::utils::utils::Utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Service {
    id: EntityId,
    name: String,
    status: ServiceStatus,
    start_node: String,
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
    pub fn new(task: &Task) -> Result<Service, CloudError> {
        let port = match Address::find_next_port(&mut Address::new(
            &CloudConfig::get().get_server_host(),
            &task.get_start_port(),
        )) {
            Some(port) => port,
            None => return Err(error!(NextFreePortNotFound)),
        };
        let server_address = Address::new(&CloudConfig::get().get_server_host(), &port);
        let service_path = task.prepared_to_service()?;
        let service = Service {
            id: Uuid::new_v4(),
            name: Directory::get_last_folder_name(&service_path),
            status: ServiceStatus::Stopped,
            start_node: CloudConfig::get().get_name(),
            current_players: 0,
            started_at: None,
            stopped_at: None,
            idle_since: None,
            server_listener: server_address,
            plugin_listener: Address::get_local_ipv4(),
            cloud_listener: CloudConfig::get().get_node_host(),
            task: task.clone(),
        };

        service.save_to_file();
        Ok(service)
    }

    pub fn get_id(&self) -> Uuid {
        self.id.clone()
    }

    pub fn is_local(&self) -> bool {
        self.start_node == CloudConfig::get().get_name()
    }

    pub fn get_start_node(&self) -> String {
        self.start_node.to_string()
    }

    pub fn set_start_node(&mut self, node: &String) {
        self.start_node = node.to_string();
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &String) {
        self.name = name.clone();
        self.save_to_file();
    }

    pub fn get_status(&self) -> ServiceStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: ServiceStatus) {
        self.status = status;
    }

    pub fn get_current_players(&self) -> u32 {
        self.current_players
    }

    pub fn update_current_player(&mut self, count: u32) {
        self.current_players = count;
    }

    pub fn get_started_at(&self) -> Option<NaiveDateTime> {
        self.started_at
    }

    pub fn get_started_at_to_string(&self) -> Option<String> {
        if let Some(date) = self.started_at {
            return Some(date.format("%Y-%m-%d %H:%M:%S").to_string())
        }
        None
    }

    pub fn get_stopped_at(&self) -> Option<NaiveDateTime> {
        self.stopped_at
    }

    pub fn get_stopped_at_to_string(&self) -> Option<String> {
        if let Some(date) = self.stopped_at {
            return Some(date.format("%Y-%m-%d %H:%M:%S").to_string())
        }
        None
    }

    pub fn get_idle_since(&self) -> Option<NaiveDateTime> {
        self.idle_since
    }

    pub fn get_idle_since_to_string(&self) -> Option<String> {
        if let Some(date) = self.stopped_at {
            return Some(date.format("%Y-%m-%d %H:%M:%S").to_string())
        }
        None
    }

    pub fn start_idle_timer(&mut self) {
        self.idle_since = Some(Utc::now().naive_utc());
    }

    pub fn get_task(&self) -> Task {
        self.task.clone()
    }

    pub fn get_software_name(&self) -> SoftwareName {
        self.get_task().get_software().get_software_name()
    }

    pub fn get_plugin_listener(&self) -> Address {
        self.plugin_listener.clone()
    }

    pub fn set_plugin_listener(&mut self, address: &Address) {
        self.plugin_listener = address.clone();
        self.save_to_file();
    }

    pub fn get_cloud_listener(&self) -> Address {
        self.cloud_listener.clone()
    }

    pub fn set_cloud_listener(&mut self, address: Address) {
        self.cloud_listener = address;
        self.save_to_file()
    }

    pub fn get_server_listener(&self) -> Address {
        self.server_listener.clone()
    }

    pub async fn set_server_listener(&mut self, manager: &ServiceManager) -> Result<(), CloudError> {
        let address = self.find_free_server_address(manager).await;

        let software_name = self.get_software_name();
        let path = self.get_path();

        // replace ip1
        let path_ip = path.join(software_name.get_ip_path());

        if !path_ip.exists() {
            return Err(error!(CantFindIPConfigFilePath));
        }

        let file_content_ip =
            read_to_string(&path_ip).map_err(|e| error!(CantReadFileToString, e))?;
        let edit_file_ip = file_content_ip.replace("%ip%", &*address.get_ip());
        fs::write(&path_ip, edit_file_ip).map_err(|e| error!(CantWriteIP, e))?;

        // replace port
        let path_port = path.join(software_name.get_port_path());

        if !path_port.exists() {
            return Err(error!(CantFindPortConfigFilePath));
        }

        let file_content_port =
            read_to_string(&path_port).map_err(|e| error!(CantReadFileToString, e))?;
        let edit_file_port =
            file_content_port.replace("%port%", address.get_port().to_string().as_str());
        fs::write(&path_port, edit_file_port).map_err(|e| error!(CantWritePort, e))?;

        self.server_listener = address;
        self.save_to_file();

        Ok(())
    }

    pub fn is_delete(&self) -> bool {
        !self.get_task().is_static_service() && self.get_task().is_delete_on_stop()
    }

    pub fn delete_files(&self) {
        if self.is_delete() {
            if fs::remove_dir_all(self.get_path()).is_err() {
                log_warning!("Service | {} | folder can't delete", self.get_name());
            }
        }
    }

    pub async fn find_free_server_address(&self, manager: &ServiceManager) -> Address {
        let ports = manager.get_bind_ports().await;
        let port = self.get_task().get_start_port();
        let server_host = manager.get_config().get_server_host();
        Address::new(&server_host, &find_port(ports, port, &server_host))
    }

    pub async fn find_free_plugin_address(&self, manager: &ServiceManager) -> Address {
        let ports = manager.get_bind_ports().await;
        let port = self.get_server_listener().get_port() + 1;
        let server_host = manager.get_config().get_server_host();
        Address::new(&server_host, &find_port(ports, port, &server_host))
    }

    pub fn get_path(&self) -> PathBuf {
        self.get_task().get_service_path().join(self.get_name())
    }

    pub fn get_path_with_server_file(&self) -> PathBuf {
        self.get_path()
            .join(self.get_task().get_software().get_server_file_name())
    }

    pub async fn find_new_free_plugin_listener(&mut self, manager: &ServiceManager) {
        let address = self.find_free_plugin_address(&manager).await;
        self.set_plugin_listener(&address);
        self.save_to_file()
    }

    pub fn get_path_with_service_config(&self) -> PathBuf {
        self.get_path().join(".minecloud")
    }

    pub fn get_path_with_service_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("service_config.json")
    }

    pub fn get_from_path(path: &mut PathBuf) -> Option<Service> {
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

    pub fn get_path_stdout_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stdout.log")
    }

    pub fn get_path_stdin_file(&self) -> PathBuf {
        self.get_path_with_service_config().join("server_stdin.log")
    }

    pub fn get_path_stderr_file(&self) -> PathBuf {
        self.get_path_with_service_config()
            .join("server_stderr.log")
    }

    pub fn save_to_file(&self) {
        let path = self.get_path_with_service_config();
        fs::create_dir_all(&path).expect("Cant create Service File in 'save_to_file'");
        if File::create(self.get_path_with_service_file()).is_err() {
            log_error!("Error by create to service config file");
            return;
        }

        if let Ok(serialized) = serde_json::to_string_pretty(&self) {
            if let Ok(mut file) = File::create(self.get_path_with_service_file()) {
                file.write_all(serialized.as_bytes())
                    .expect("Error by save the service config file");
            }
        }
    }

    pub fn is_start(&self) -> bool {
        self.status == ServiceStatus::Starting ||
            self.status == ServiceStatus::Running
    }
    pub fn is_stop(&self) -> bool {
        self.status == ServiceStatus::Stopped ||
            self.status == ServiceStatus::Stopping ||
            self.status == ServiceStatus::Failed
    }

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

    pub fn is_service_start_or_prepare(path: &mut PathBuf) -> bool {
        match Service::get_from_path(path) {
            Some(service) => service.is_start() || !service.is_stop(),
            None => false,
        }
    }



    pub fn get_service_url(&self) -> Url {
        Url::new(
            UrlSchema::Http,
            &self.get_plugin_listener(),
            "cloud/service",
        )
        .join(&self.get_name())
    }

    pub fn get_from_name(name: &String) -> Option<Service> {
        let mut path = CloudConfig::get()
            .get_cloud_path()
            .get_service_folder()
            .get_temp_folder_path()
            .join(&name);
        Service::get_from_path(&mut path)
    }

    pub fn install_software(&self) -> Result<(), CloudError> {
        let target_path = self
            .get_path()
            .join(&self.get_task().get_software().get_server_file_name());
        let software_path = self.get_task().get_software().get_software_file_path();

        fs::copy(&software_path, &target_path).map_err(|e| error!(CantCopySoftware, e))?;
        Ok(())
    }

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

    pub fn is_proxy(&self) -> bool {
        self.get_software_name().get_server_type().is_proxy()
    }

    pub fn is_backend_server(&self) -> bool {
        self.get_software_name()
            .get_server_type()
            .is_backend_server()
    }
}

fn find_port(ports: Vec<u32>, mut port: u32, server_host: &String) -> u32 {
    while ports.contains(&port) || !Address::is_port_available(&Address::new(&server_host, &port)) {
        port = Address::find_next_port(&mut Address::new(&server_host, &(port + 1)))
            .unwrap_or_else(|| 0);
    }
    port
}
