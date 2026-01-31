use std::io::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use bx::path::Directory;
use serde::Serialize;
use serde_json::json;
use tokio::sync::RwLock;

use crate::api::internal::node_service::ServiceInfoResponse;
use crate::config::cloud_config::CloudConfig;
use crate::config::software_config::SoftwareConfig;
use crate::database::manager::DatabaseManager;
use crate::{error, log_error, log_info, log_warning};
use crate::manager::process::ServiceProcess;
use crate::types::{EntityId, ServiceStatus};
use crate::types::service::Service;
use crate::types::task::Task;
use crate::utils::error::cloud_error::CloudError;
use crate::utils::error::error_kind::CloudErrorKind::*;
use crate::utils::utils::Utils;


#[derive(Serialize)]
struct RegisterServerData {
    register_server: ServiceInfoResponse,
}


pub struct ServiceManager {
    services: Vec<ServiceProcess>,
    _db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    _software_config: Arc<RwLock<SoftwareConfig>>,
}

impl ServiceManager {
    pub fn new(_db: Arc<DatabaseManager>, cloud_config: Arc<CloudConfig>, _software_config: Arc<RwLock<SoftwareConfig>>) -> Self {
        let service = Self::get_all_from_file();
        let mut services: Vec<ServiceProcess> = Vec::new();
        for s in service {
            services.push(ServiceProcess::new(s));
        }
        Self {
            services,
            _db,
            config: cloud_config,
            _software_config,
        }
    }

    pub fn create_service(&self, task: &Task) -> Result<ServiceProcess, CloudError> {
        if !task.is_startup_local(&self.config) {
            todo!("Network");
        }

        let mut service = match self.get_next_stopped_service(task) {
            Some(s) => s,
            None => ServiceProcess::create(task)?,
        };

        service.get_service_mut().set_status(ServiceStatus::Starting);
        service.get_service_mut().install_software()?;
        service.get_service_mut().install_system_plugin()?;
        service
            .get_service_mut()
            .install_software_lib(&self.config)?;
        service.get_service_mut().set_server_address(self)?;
        service.get_service_mut().find_new_free_plugin_listener(self);

        Ok(service)
    }


    pub fn start_service(&self, task: &Task) -> Result<ServiceProcess, CloudError> {
        if !task.is_startup_local(&self.config) {
            todo!("Network");
        }

        let mut service = match self.get_next_stopped_service(&task) {
            Some(s) =>  s,
            None => ServiceProcess::create(task)?,
        };

        service.get_service_mut().set_status(ServiceStatus::Starting);
        service.get_service_mut().install_software()?;
        service
            .get_service_mut()
            .install_software_lib(&self.get_config())?;
        service.get_service_mut().set_server_address(&self)?;
        service.get_service_mut().find_new_free_plugin_listener(&self);

        service.start()?;
        Ok(service)
    }

    pub async fn connect_to_network(&self, service: &Service) -> Result<(), Error> {
        if service.is_proxy() {
            // TODO: Send New Started Proxy Service To Cluster
            return Ok(());
        }

        for service_proxy in self.get_online_proxies() {
            let s = service_proxy.get_service();
            let url = s.get_service_url().join("add_server");
            let body = match Utils::convert_to_json(&RegisterServerData {
                register_server: ServiceInfoResponse::new(service),
            }) {
                Some(body) => body,
                None => {
                    log_warning!("Service {} can't Serialize to ServiceInfo", service.get_name());
                    continue;
                }
            };

            match url.post(&body, Duration::from_secs(3)).await {
                Ok(_) => log_info!(
                    "Service {} successfully connected to Proxy [{}]",
                    service.get_name(),
                    s.get_name()
                ),
                Err(e) => log_warning!(
                    "Service | {} | can't send request connect to Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                ),
            }
        }

        // TODO: Send New Started Service To Cluster
        Ok(())
    }

    pub async fn disconnect_from_network(&self, service: &Service) -> Result<(), Error> {
        if service.is_proxy() {
            return Ok(());
            // TODO: Send New Stopped Proxy Service To Cluster
        }

        for service_proxy in self.get_online_proxies() {
            let s = service_proxy.get_service();
            let url = s
                .get_service_url()
                .join(format!("remove_server?name={}", service.get_name()).as_str());
            match url.post(&json!({}), Duration::from_secs(3)).await {
                Ok(_) => log_info!(
                    "Service {} successfully disconnected from Proxy [{}]",
                    service.get_name(),
                    s.get_name()
                ),
                Err(e) => log_warning!(
                    "Service | {} | can't send request disconnect from Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                ),
            }
        }
        // TODO: Send New Stopped Service To Cluster
        Ok(())
    }

    pub fn set_service(&mut self, service: ServiceProcess) {
        if let Some(pos) = self.services.iter().position(|s| s.get_service().get_id() == service.get_service().get_id()) {
            self.services.remove(pos);
        }
        self.services.push(service);
    }

    pub fn get_next_stopped_service(
        &self,
        task: &Task,
    ) -> Option<ServiceProcess> {
        self.services
            .iter()
            .map(|s| s.clone_without_process())
            .find(|s| s.get_service().is_stop() && s.get_service().get_task() == *task)
    }

    pub async fn stop_all(&mut self, shutdown_msg: &str) {
        let ids: Vec<EntityId> = self.get_all().iter().map(|s| s.get_service().get_id()).collect();
        for id in ids {
            match self.stop_service(&id, shutdown_msg).await {
                Ok(_) => log_info!(3, "Service successfully shutdown"),
                Err(e) => log_error!(2, "Service Cant shutdown with Error: {}", e)
            }
        }
    }

    pub async fn stop_service(&mut self, id: &EntityId, shutdown_msg: &str) -> Result<(), CloudError> {
        let (pos, process) = match self.get_from_id_mut(id) {
            Some(process) => process,
            None => return Err(error!(CantFindServiceFromUUID)),
        };

        process.shutdown(shutdown_msg).await?;
        self.remove_service(pos);

        Ok(())
    }

    pub fn remove_service(&mut self, pos: usize) {
        let s = self.services.get_mut(pos).unwrap();
        if s.get_service().is_delete() {
            s.get_service().delete_files();
            self.services.remove(pos);
        }
        else {
            s.get_service_mut().set_status(ServiceStatus::Stopped);
            s.get_service().save_to_file();
        }
    }

    pub fn get_config(&self) -> &Arc<CloudConfig> {
        &self.config
    }

    pub fn get_from_id_mut(&mut self, id: &EntityId) -> Option<(usize, &mut ServiceProcess)> {
        if let Some(pos) = self.services.iter().position(|s| s.get_service().get_id() == *id) {
            if let Some(service) = self.services.get_mut(pos) {
                return Some((pos, service))
            }
        }
        None
    }

    pub fn get_from_id(&self, id: &EntityId) -> Option<ServiceProcess> {
        self.get_all().into_iter().find(|s| s.get_service().get_id() == *id)
    }

    pub fn get_all(&self) -> Vec<ServiceProcess> {
        self.services.iter().map(|s| s.clone_without_process()).collect()
    }

    pub fn get_all_from_task(&self, task_name: &str) -> Vec<ServiceProcess> {
        self.services
            .iter()
            .filter(|s| s.get_service().get_task().get_name() == task_name)
            .map(|s| s.clone_without_process())
            .collect()
    }

    pub fn get_online_proxies(&self) -> Vec<ServiceProcess> {
        self.services
            .iter()
            .filter(|process| {
                let service = process.get_service();

                service.is_start()
                    && service.is_proxy()
            })
            .map(|s| s.clone_without_process())
            .collect()
    }

    pub fn get_online_backend_server(&self) -> Vec<ServiceProcess> {
        self.services
            .iter()
            .filter(|process| {
                let service = process.get_service();

                service.is_start()
                    && service.is_backend_server()
            })
            .map(|s| s.clone_without_process())
            .collect()
    }

    pub fn get_bind_ports(
        &self,
    ) -> Vec<u32> {
        let host = self.config.get_server_host();
        let mut ports = Vec::new();

        for process in &self.services {
            let service = process.get_service();

            let server_listener = service.get_server_listener();
            if server_listener.get_ip() == host {
                ports.push(server_listener.get_port());
            }

            let plugin_listener = service.get_plugin_listener();
            if plugin_listener.get_ip() == host {
                ports.push(plugin_listener.get_port());
            }
        }

        ports
    }

    pub fn get_all_from_file() -> Vec<Service> {
        let mut service_list: Vec<Service> = Vec::new();
        service_list.append(&mut get_services_from_path(
            &CloudConfig::get()
                .get_cloud_path()
                .get_service_folder()
                .get_temp_folder_path(),
        ));

        service_list.append(&mut get_services_from_path(
            &CloudConfig::get()
                .get_cloud_path()
                .get_service_folder()
                .get_static_folder_path(),
        ));

        service_list
    }

}

fn get_services_from_path(path: &PathBuf) -> Vec<Service> {
    let mut service_list: Vec<Service> = Vec::new();
    for folder in Directory::get_folders_name_from_path(&path) {
        let mut path = path.clone();
        path.push(folder);
        if let Some(service) = Service::get_from_path(&mut path) {
            service_list.push(service);
        };
    }
    service_list
}


