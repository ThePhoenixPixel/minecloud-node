use std::fmt::Debug;
use std::io::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use bx::network::address::Address;
use bx::path::Directory;
use database_manager::DatabaseManager;
use serde::Serialize;
use serde_json::json;
use tokio::sync::RwLock;

use crate::api::internal::node_service::ServiceInfoResponse;
use crate::config::cloud_config::CloudConfig;
use crate::config::software_config::SoftwareConfig;
use crate::{error, log_error, log_info, log_warning};
use crate::database::table::TableServices;
use crate::types::process::ServiceProcess;
use crate::types::{EntityId, ServiceRef, ServiceStatus};
use crate::types::service::Service;
use crate::types::task::Task;
use crate::utils::error::*;
use crate::utils::utils::Utils;


#[derive(Serialize)]
struct RegisterServerData {
    register_server: ServiceInfoResponse,
}


pub struct ServiceManager {
    services: Vec<ServiceRef>,
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    _software_config: Arc<RwLock<SoftwareConfig>>,
}

impl ServiceManager {
    pub async fn new(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        _software_config: Arc<RwLock<SoftwareConfig>>
    ) -> CloudResult<Self> {
        let service = Self::get_all_from_file();
        TableServices::delete_others(db.as_ref(), &service, cloud_config.as_ref()).await?;
        let mut services: Vec<ServiceRef> = Vec::new();
        for s in service {
            services.push(ServiceRef::new(ServiceProcess::new(s)));
        }
        Ok(Self {
            services,
            db,
            config: cloud_config,
            _software_config,
        })
    }

    pub async fn start(&self, service: ServiceRef) -> CloudResult<()> {
        {
            let mut s = service.write().await;
            s.set_status(ServiceStatus::Starting);
        } // write guard gedroppt

        TableServices::update(self.get_db(), &service).await?;

        self.prepare_to_start(&service).await?;
        TableServices::update(self.get_db(), &service).await?;

        service.write().await.start()?;
        TableServices::update(self.get_db(), &service).await?;

        Ok(())
    }

    pub async fn get_or_create_service_ref(&mut self, task: &Task) -> CloudResult<ServiceRef> {
        match self.get_next_stopped_service(task).await {
            Some(arc) => {
                TableServices::update(self.get_db(), &arc).await?;
                Ok(arc)
            }
            None => {
                let s = ServiceProcess::create(task)?;
                let arc = ServiceRef::new(s);
                TableServices::create(self.get_db(), &arc).await?;
                self.services.push(arc.clone());
                Ok(arc)
            }
        }
    }

    async fn prepare_to_start(&self, service: &ServiceRef) -> CloudResult<()> {
        // FIX: read guard in eigenem Scope, damit er vor write() gedroppt wird
        {
            let s = service.read().await;
            s.install_software()?;
            s.install_system_plugin()?;
            s.install_software_lib(self.get_config())?;
        } // read guard gedroppt

        // FIX: Ports VOR dem write-Lock sammeln, weil set_server_listener intern
        // get_bind_ports() aufruft → würde denselben Service read()-locken → Deadlock
        let bind_ports = self.get_bind_ports().await;

        {
            let mut s = service.write().await;
            s.set_server_listener_with_ports(&bind_ports, self.get_config()).await?;
            s.find_new_free_plugin_listener_with_ports(&bind_ports, self.get_config()).await;
        }

        Ok(())
    }

    pub async fn connect_to_network(&self, service: &Service) -> CloudResult<()> {
        for service_proxy in self.get_online_proxies().await {
            let s = service_proxy.read().await;
            let proxy_service = s.get_service();
            let url = proxy_service.get_service_url().join("add_server");
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
                    proxy_service.get_name()
                ),
                Err(e) => log_warning!(
                    "Service | {} | can't send request connect to Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                ),
            }
        }
        Ok(())
    }

    pub async fn disconnect_from_network(&self, service: &Service) -> Result<(), Error> {
        for service_proxy in self.get_online_proxies().await {
            let s = service_proxy.read().await;
            let proxy_service = s.get_service();
            let url = proxy_service
                .get_service_url()
                .join(format!("remove_server?name={}", service.get_name()).as_str());
            match url.post(&json!({}), Duration::from_secs(3)).await {
                Ok(_) => log_info!(
                    "Service {} successfully disconnected from Proxy [{}]",
                    service.get_name(),
                    proxy_service.get_name()
                ),
                Err(e) => log_warning!(
                    "Service | {} | can't send request disconnect from Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                ),
            }
        }
        Ok(())
    }

    // FIX: Arc<RwLock<ServiceProcess>> -> ServiceRef
    pub async fn set_service(&mut self, service: ServiceRef) {
        let id = service.read().await.get_service().get_id();
        if let Some(pos) = self.find_pos_by_id(&id).await {
            self.services[pos] = service;
        } else {
            self.services.push(service);
        }
    }

    async fn get_next_stopped_service(&self, task: &Task) -> Option<ServiceRef> {
        for arc in &self.services {
            let p = arc.read().await;
            if p.get_service().is_stop() && p.get_service().get_task() == *task {
                return Some(arc.clone());
            }
        }
        None
    }

    pub async fn stop_all(&mut self, shutdown_msg: &str) {
        let ids: Vec<EntityId> = {
            let mut ids = Vec::new();
            for arc in &self.services {
                ids.push(arc.read().await.get_service().get_id());
            }
            ids
        };
        for id in ids {
            match self.stop_service(&id, shutdown_msg).await {
                Ok(_) => log_info!(3, "Service successfully shutdown"),
                Err(e) => log_error!(2, "Service Cant shutdown with Error: {}", e),
            }
        }
    }

    pub async fn stop_service(&mut self, id: &EntityId, shutdown_msg: &str) -> CloudResult<()> {
        let pos = match self.find_pos_by_id(id).await {
            Some(pos) => pos,
            None => return Err(error!(CantFindServiceFromUUID)),
        };

        self.services[pos].write().await.shutdown(shutdown_msg).await?;
        self.remove_service(pos).await;

        Ok(())
    }

    pub async fn remove_service(&mut self, pos: usize) {
        let arc = self.services[pos].clone();
        let mut p = arc.write().await;

        if p.get_service().is_delete() {
            p.get_service().delete_files();
            drop(p);
            self.services.remove(pos);
        } else {
            p.set_status(ServiceStatus::Stopped);
            p.save_to_file();
        }
    }

    pub fn get_config(&self) -> &Arc<CloudConfig> {
        &self.config
    }

    pub async fn get_from_id(&self, id: &EntityId) -> Option<ServiceRef> {
        for arc in &self.services {
            if arc.read().await.get_service().get_id() == *id {
                return Some(arc.clone());
            }
        }
        None
    }

    pub async fn get_all_from_task(&self, task_name: &str) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            if arc.read().await.get_service().get_task().get_name() == task_name {
                result.push(arc.clone());
            }
        }
        result
    }

    pub async fn get_online_all_from_task(&self, task_name: &str) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            let p = arc.read().await;
            if p.get_service().get_task().get_name() == task_name && p.is_start() {
                result.push(arc.clone());
            }
        }
        result
    }

    pub async fn get_online_all(&self) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            if arc.read().await.is_start() {
                result.push(arc.clone());
            }
        }
        result
    }

    pub async fn get_online_proxies(&self) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            let p = arc.read().await;
            if p.is_start() && p.get_service().is_proxy() {
                result.push(arc.clone());
            }
        }
        result
    }

    pub async fn get_online_backend_server(&self) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            let p = arc.read().await;
            if p.is_start() && p.get_service().is_backend_server() {
                result.push(arc.clone());
            }
        }
        result
    }

    async fn set_server_listener(&self, service: &ServiceRef) {



    }

    async fn set_plugin_listener(&self, service: &ServiceRef) {



    }


    async fn get_used_ports(&self) -> Vec<u32> {
        let host = self.config.get_server_host();
        let mut ports = Vec::new();

        for arc in &self.services {
            let service = arc.read().await;
            if service.is_start() {
                continue;
            }

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

    #[deprecated]
    pub async fn get_bind_ports(&self) -> Vec<u32> {
        let host = self.config.get_server_host();
        let mut ports = Vec::new();

        for arc in &self.services {
            let service = arc.read().await;
            if service.is_start() {
                continue;
            }

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
            &CloudConfig::get().get_cloud_path().get_service_folder().get_temp_folder_path(),
        ));
        service_list.append(&mut get_services_from_path(
            &CloudConfig::get().get_cloud_path().get_service_folder().get_static_folder_path(),
        ));
        service_list
    }


    async fn find_pos_by_id(&self, id: &EntityId) -> Option<usize> {
        for (pos, arc) in self.services.iter().enumerate() {
            if arc.read().await.get_service().get_id() == *id {
                return Some(pos);
            }
        }
        None
    }

    fn get_db(&self) -> &DatabaseManager {
        self.db.as_ref()
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