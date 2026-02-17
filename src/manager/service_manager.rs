use std::fs;
use std::fs::read_to_string;
use std::io::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use bx::network::address::Address;
use bx::path::Directory;
use database_manager::DatabaseManager;
use serde::Serialize;
use serde_json::json;

use crate::api::internal::node_service::ServiceInfoResponse;
use crate::{error, log_error, log_info, log_warning};
use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::database::table::TableServices;
use crate::types::{EntityId, Service, ServiceProcess, ServiceRef, ServiceStatus, Task};
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
    _software_config: SoftwareConfigRef,
}

impl ServiceManager {
    pub async fn new(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        _software_config: SoftwareConfigRef
    ) -> CloudResult<Self> {
        let mut service = Self::get_all_from_file();
        service.iter_mut().for_each(|s| s.set_status(ServiceStatus::Stopped));
        service.iter_mut().for_each(|s| s.save_to_file());
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
        {
            let s = service.read().await;
            s.install_software()?;
            s.install_system_plugin()?;
            s.install_software_lib(self.get_config())?;
        }

        self.set_server_listener(service).await?;
        self.set_plugin_listener(service).await;

        Ok(())
    }

    pub async fn connect_to_network(&self, service: &Service) -> CloudResult<()> {
        for service_proxy in self.get_online_proxies().await {
            let s = service_proxy.read().await;
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
        Ok(())
    }

    pub async fn disconnect_from_network(&self, service: &Service) -> Result<(), Error> {
        for service_proxy in self.get_online_proxies().await {
            let s = service_proxy.read().await;
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
        Ok(())
    }

    // FIX: Arc<RwLock<ServiceProcess>> -> ServiceRef
    pub async fn set_service(&mut self, service: ServiceRef) {
        let id = service.get_id().await;
        if let Some(pos) = self.find_pos_by_id(&id).await {
            self.services[pos] = service;
        } else {
            self.services.push(service);
        }
    }

    async fn get_next_stopped_service(&self, task: &Task) -> Option<ServiceRef> {
        for arc in &self.services {
            let p = arc.read().await;
            if p.is_stop() && p.get_task() == *task {
                return Some(arc.clone());
            }
        }
        None
    }

    pub async fn stop_all(&mut self, shutdown_msg: &str) {
        let ids: Vec<EntityId> = {
            let mut ids = Vec::new();
            for arc in &self.services {
                ids.push(arc.get_id().await);
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

        if p.is_delete() {
            p.delete_files();
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
            if arc.read().await.get_id() == *id {
                return Some(arc.clone());
            }
        }
        None
    }

    pub async fn get_all_from_task(&self, task_name: &str) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            if arc.read().await.get_task().get_name() == task_name {
                result.push(arc.clone());
            }
        }
        result
    }

    pub async fn get_online_all_from_task(&self, task_name: &str) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            let p = arc.read().await;
            if p.get_task().get_name() == task_name && p.is_start() {
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
            if p.is_start() && p.is_proxy() {
                result.push(arc.clone());
            }
        }
        result
    }

    pub async fn get_online_backend_server(&self) -> Vec<ServiceRef> {
        let mut result = Vec::new();
        for arc in &self.services {
            let p = arc.read().await;
            if p.is_start() && p.is_backend_server() {
                result.push(arc.clone());
            }
        }
        result
    }

    async fn set_server_listener(&self, service: &ServiceRef) -> CloudResult<()> {
        let bind_ports = self.get_bind_ports_except(service).await;

        let mut s = service.write().await;
        let start_port = s.get_task().get_start_port();
        let host = self.config.get_server_host();
        let port = Utils::find_free_port(&bind_ports, start_port, &host);
        let address = Address::new(&host, &port);

        let software_name = s.get_software_name();
        let path = s.get_path();

        let path_ip = path.join(software_name.get_ip_path());
        if !path_ip.exists() {
            return Err(error!(CantFindIPConfigFilePath));
        }
        let content = read_to_string(&path_ip).map_err(|e| error!(CantReadFileToString, e))?;
        fs::write(&path_ip, content.replace("%ip%", &address.get_ip()))
            .map_err(|e| error!(CantWriteIP, e))?;

        let path_port = path.join(software_name.get_port_path());
        if !path_port.exists() {
            return Err(error!(CantFindPortConfigFilePath));
        }
        let content = read_to_string(&path_port).map_err(|e| error!(CantReadFileToString, e))?;
        fs::write(&path_port, content.replace("%port%", &address.get_port().to_string()))
            .map_err(|e| error!(CantWritePort, e))?;

        s.set_server_listener(address);
        s.save_to_file();
        Ok(())
    }

    async fn set_plugin_listener(&self, service: &ServiceRef) {
        let bind_ports = self.get_bind_ports_except(service).await;

        let mut s = service.write().await;
        let start_port = s.get_server_listener().get_port() + 1;
        let host = self.config.get_server_host();
        let port = Utils::find_free_port(&bind_ports, start_port, &host);
        let address = Address::new(&host, &port);

        s.set_plugin_listener(address);
        s.save_to_file();
    }

    async fn get_bind_ports_except(&self, exclude: &ServiceRef) -> Vec<u32> {
        let host = self.config.get_server_host();
        let mut ports = Vec::new();

        for arc in &self.services {
            if arc.ptr_eq(exclude) {
                continue;
            }

            let service = arc.read().await;
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
            if arc.get_id().await == *id {
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