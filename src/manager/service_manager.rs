use bx::network::address::Address;
use bx::path::Directory;
use database_manager::DatabaseManager;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use uuid::Uuid;

use crate::api::internal::ServiceInfoResponse;
use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::database::table::TableServices;
use crate::manager::TaskManagerRef;
use crate::types::{EntityId, Service, ServiceProcess, ServiceProcessRef, ServiceStatus, TaskRef};
use crate::utils::error::*;
use crate::utils::utils::Utils;
use crate::{error, log_info, log_warning};

#[derive(Serialize)]
struct RegisterServerData {
    register_server: ServiceInfoResponse,
}

pub struct ServiceManager {
    services: HashMap<EntityId, ServiceProcessRef>,
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    task_manager: TaskManagerRef,
    _software_config: SoftwareConfigRef,
}

pub struct ServiceManagerRef(Arc<RwLock<ServiceManager>>);

impl ServiceManager {
    pub async fn create_service(&mut self, task_ref: &TaskRef) -> CloudResult<ServiceProcessRef> {
        let (name, split, task) = {
            let t = task_ref.read().await;
            (t.get_name(), t.get_split(), t.clone())
        };

        let next_free_number =
            TableServices::find_next_free_number(self.get_db(), task_ref).await?;
        let id = Uuid::new_v4();
        let name = format!("{}{}{}", name, split, next_free_number);
        let path = {
            let tm = self.task_manager.read().await;
            tm.get_service_path(task_ref).await.join(&name)
        };

        let service = Service::new(id, name, &task, &self.config);
        let sp = ServiceProcessRef::new(service, path);
        self.task_manager.read().await.prepared_to_service(&sp).await?;

        // Insert in Database
        TableServices::create_if_not_exists(self.get_db(), &sp).await?;

        // insert in local List
        self.services.insert(id, sp.clone());

        Ok(sp)
    }

    pub async fn get_or_create_service(
        &mut self,
        task_ref: &TaskRef,
    ) -> CloudResult<ServiceProcessRef> {
        let task_name = task_ref.get_name().await;
        let s = self
            .filter_services(|sp| sp.is_stop() && sp.get_task_name() == task_name)
            .await;
        if let Some(sp) = s.first() {
            return Ok(sp.clone());
        }
        // Create a new local service
        self.create_service(task_ref).await
    }

    pub async fn start(&self, service_ref: ServiceProcessRef) -> CloudResult<()> {
        let service = {
            let mut s = service_ref.write().await;
            s.set_status(ServiceStatus::Starting);
            s.get_service().clone()
        };

        TableServices::update(self.get_db(), &service).await?;

        self.prepare_to_start(&service_ref).await?;
        TableServices::update(self.get_db(), &service).await?;

        service_ref.write().await.start()?;
        TableServices::update(self.get_db(), &service).await?;

        Ok(())
    }

    async fn prepare_to_start(&self, service: &ServiceProcessRef) -> CloudResult<()> {
        {
            let s = service.read().await;
            s.install_software()?;
            s.install_system_plugin()?;
            s.install_software_lib(&self.config)?;
        }

        self.set_server_listener(service).await?;
        self.set_plugin_listener(service).await;

        Ok(())
    }

    pub async fn stop_service(
        &mut self,
        service_process_ref: &ServiceProcessRef,
        shutdown_msg: &str,
    ) {
        let (id, task_name) = {
            let sp = service_process_ref.read().await;
            (sp.get_id().clone(), sp.get_task_name().to_string())
        };

        {
            let mut sp = service_process_ref.write().await;
            sp.shutdown(shutdown_msg).await;
        }

        match self.task_manager.get_task_ref_from_name(&task_name).await {
            Ok(task_ref) => {
                let should_delete = task_ref.read().await.is_delete();

                // Cleanup
                if should_delete {
                    service_process_ref.read().await.delete_files();
                    if let Err(e) = TableServices::delete(self.get_db(), &id).await {
                        log_warning!("Error by deleting Service {}  in Database: {:?}", id, e);
                    }

                    // Aus ServiceManager entfernen
                    self.services.remove(&id);
                } else {
                    // Update in DB
                    let service = service_process_ref.read().await.get_service().clone();
                    if let Err(e) = TableServices::update(self.get_db(), &service).await {
                        log_warning!(
                            "Fehler beim Aktualisieren von Service {} in DB: {:?}",
                            id,
                            e
                        );
                    }
                }
            }

            Err(_) => {
                log_info!(
                    "Service {} gehört zu Task {}, die nicht mehr existiert. Lösche Service.",
                    id,
                    task_name
                );
                service_process_ref.read().await.delete_files();
                if let Err(e) = TableServices::delete(self.get_db(), &id).await {
                    log_warning!("Error by deleting Service {}  in Database: {:?}", id, e);
                }

                self.services.remove(&id);
            }
        }
    }

    pub async fn register_on_proxy(&self, service: &Service) -> CloudResult<()> {
        if service.is_proxy() {
            return Ok(());
        }

        for proxy in self.get_online_proxies().await {
            let s = proxy.read().await;
            let url = s.get_service_url().join("add_server");
            let body = match Utils::convert_to_json(&RegisterServerData {
                register_server: ServiceInfoResponse::new(&service),
            }) {
                Some(b) => b,
                None => {
                    log_warning!(
                        2,
                        "Service [{}] can't Serialize to ServiceInfo",
                        service.get_name()
                    );
                    continue;
                }
            };

            match url.post(&body, Duration::from_secs(3)).await {
                Ok(_) => log_info!(
                    4,
                    "Successfully connect Service [{}] to Proxy [{}] ",
                    service.get_name(),
                    s.get_name()
                ),
                Err(e) => log_warning!(
                    2,
                    "Can't Register Service [{}] to Proxy [{}] -> {}",
                    service.get_name(),
                    s.get_name(),
                    e
                ),
            }
        }
        Ok(())
    }

    pub async fn unregister_from_proxy(&self, service: &Service) -> CloudResult<()> {
        if service.is_proxy() {
            return Ok(());
        }

        for proxy in self.get_online_proxies().await {
            let s = proxy.read().await;
            let url = s
                .get_service_url()
                .join(format!("remove_server?name={}", service.get_name()).as_str());

            match url.post(&json!({}), Duration::from_secs(3)).await {
                Ok(_) => log_info!(
                    4,
                    "Successfully disconnect Service [{}] from Proxy [{}] ",
                    service.get_name(),
                    s.get_name()
                ),
                Err(e) => log_warning!(
                    2,
                    "Can't Unregister Service [{}] from Proxy [{}] -> {}",
                    service.get_name(),
                    s.get_name(),
                    e
                ),
            }
        }
        Ok(())
    }

    pub fn get_from_id(&self, id: &EntityId) -> CloudResult<ServiceProcessRef> {
        self.find_from_id(id).ok_or(error!(CantFindServiceFromUUID))
    }

    pub fn find_from_id(&self, id: &EntityId) -> Option<ServiceProcessRef> {
        for (id_ref, sp_ref) in &self.services {
            if id_ref == id {
                return Some(sp_ref.clone());
            }
        }
        None
    }

    pub async fn filter_services<F>(&self, mut filter: F) -> Vec<ServiceProcessRef>
    where
        F: FnMut(&ServiceProcess) -> bool,
    {
        let mut result = Vec::new();

        for arc in self.services.values() {
            let sp = arc.read().await;

            if filter(&sp) {
                result.push(arc.clone());
            }
        }

        result
    }

    #[deprecated]
    pub async fn get_all_from_task_name(&self, task_name: &str) -> Vec<ServiceProcessRef> {
        self.filter_services(|sp| sp.get_task_name() == task_name)
            .await
    }

    #[deprecated]
    pub async fn get_online_all_from_task(&self, task_name: &str) -> Vec<ServiceProcessRef> {
        let mut result = Vec::new();

        for arc in self.services.values() {
            let sp = arc.read().await;

            if sp.get_task_name() == task_name && sp.is_start() {
                result.push(arc.clone());
            }
        }

        result
    }

    #[deprecated]
    pub async fn get_online_all(&self) -> Vec<ServiceProcessRef> {
        let mut result = Vec::new();

        for arc in self.services.values() {
            if arc.read().await.is_start() {
                result.push(arc.clone());
            }
        }

        result
    }

    #[deprecated]
    pub async fn get_online_proxies(&self) -> Vec<ServiceProcessRef> {
        let mut result = Vec::new();

        for arc in self.services.values() {
            let sp = arc.read().await;

            if sp.is_start() && sp.is_proxy() {
                result.push(arc.clone());
            }
        }

        result
    }

    #[deprecated]
    pub async fn get_online_backend_server(&self) -> Vec<ServiceProcessRef> {
        let mut result = Vec::new();

        for arc in self.services.values() {
            let sp = arc.read().await;

            if sp.is_start() && sp.is_backend_server() {
                result.push(arc.clone());
            }
        }

        result
    }

    async fn set_server_listener(&self, service: &ServiceProcessRef) -> CloudResult<()> {
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
        fs::write(
            &path_port,
            content.replace("%port%", &address.get_port().to_string()),
        )
        .map_err(|e| error!(CantWritePort, e))?;

        s.set_server_listener(address);
        s.save_to_file();
        Ok(())
    }

    async fn set_plugin_listener(&self, service: &ServiceProcessRef) {
        let bind_ports = self.get_bind_ports_except(service).await;

        let mut s = service.write().await;
        let start_port = s.get_server_listener().get_port() + 1;
        let host = self.config.get_server_host();
        let port = Utils::find_free_port(&bind_ports, start_port, &host);
        let address = Address::new(&host, &port);

        s.set_plugin_listener(address);
        s.save_to_file();
    }

    async fn get_bind_ports_except(&self, exclude: &ServiceProcessRef) -> Vec<u32> {
        let host = self.config.get_server_host();
        let mut ports = Vec::new();

        for (id, arc) in &self.services {
            if id == &exclude.get_id().await {
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

    fn get_db(&self) -> &DatabaseManager {
        self.db.as_ref()
    }
}

fn get_all_from_file() -> Vec<ServiceProcessRef> {
    let mut service_list: Vec<ServiceProcessRef> = Vec::new();
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

fn get_services_from_path(path: &PathBuf) -> Vec<ServiceProcessRef> {
    let mut service_list: Vec<ServiceProcessRef> = Vec::new();
    for folder in Directory::get_folders_name_from_path(path) {
        let mut path = path.clone();
        path.push(folder);
        if let Some(service) = get_from_path(&path) {
            service_list.push(ServiceProcessRef::new(service, path));
        };
    }
    service_list
}

fn get_from_path(path: &Path) -> Option<Service> {
    //path -> /service/temp/Lobby-1/
    let p = path.join(".minecloud").join("service_config.json");
    if let Ok(file_content) = read_to_string(p) {
        serde_json::from_str(&file_content).ok()
    } else {
        None
    }
}

impl ServiceManagerRef {
    pub async fn new(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        task_manager: TaskManagerRef,
        _software_config: SoftwareConfigRef,
    ) -> CloudResult<Self> {
        let local_services = get_all_from_file();
        TableServices::delete_others(db.as_ref(), &local_services, cloud_config.as_ref()).await?;
        let mut services: HashMap<EntityId, ServiceProcessRef> = HashMap::new();

        for sp_ref in local_services {
            {
                sp_ref.write().await.set_status(ServiceStatus::Stopped);
            }
            TableServices::create_if_not_exists(db.as_ref(), &sp_ref).await?;
            services.insert(sp_ref.get_id().await, sp_ref);
        }

        let sm = ServiceManager {
            services,
            db,
            config: cloud_config,
            task_manager,
            _software_config,
        };

        Ok(ServiceManagerRef(Arc::new(RwLock::new(sm))))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, ServiceManager> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, ServiceManager> {
        self.0.write().await
    }

    pub async fn get_service_ref_from_id(&self, id: &EntityId) -> CloudResult<ServiceProcessRef> {
        self.0.read().await.get_from_id(id)
    }
}

impl Clone for ServiceManagerRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
