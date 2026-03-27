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
    software_config: SoftwareConfigRef,
}

pub struct ServiceManagerRef(Arc<RwLock<ServiceManager>>);

impl ServiceManager {
    pub async fn create_service(&mut self, task_ref: &TaskRef) -> CloudResult<ServiceProcessRef> {
        let (name, split, task) = {
            let t = task_ref.read().await;
            (t.get_name(), t.get_split(), t.clone())
        };

        let next_free_number = TableServices::find_next_free_number(self.get_db(), task_ref).await?;
        let id = Uuid::new_v4();
        let name = format!("{}{}{}", name, split, next_free_number);
        let path = {
            let tm = self.task_manager.read().await;
            tm.get_service_path(task_ref).await.join(&name)
        };

        let service = Service::new(id, name, &task, &self.config);
        let sp = ServiceProcessRef::new(service, path);

        TableServices::create_if_not_exists(self.get_db(), &sp).await?;
        self.services.insert(id, sp.clone());

        Ok(sp)
    }

    pub async fn get_or_create_service(&mut self, task_ref: &TaskRef) -> CloudResult<ServiceProcessRef> {
        let task_name = task_ref.get_name().await;
        let s = self.filter_services(|sp| sp.is_stop() && sp.get_task_name() == task_name).await;
        if let Some(sp) = s.first() {
            return Ok(sp.clone());
        }
        self.create_service(task_ref).await
    }

    pub async fn start(&self, service_ref: ServiceProcessRef) -> CloudResult<()> {
        self.update_status(&service_ref, ServiceStatus::Starting).await;

        let service = service_ref.read().await.get_service().clone();

        self.prepare_to_start(&service_ref).await?;
        TableServices::update(self.get_db(), &service).await?;

        // Software aus Config holen für start()
        let software = {
            let software_link = service_ref.read().await.get_config().get_software().clone();
            self.software_config.get_software(&software_link).await?
        };

        service_ref.write().await.start(&software)?;
        TableServices::update(self.get_db(), &service).await?;

        Ok(())
    }

    async fn prepare_to_start(&self, service: &ServiceProcessRef) -> CloudResult<()> {
        self.task_manager.read().await.prepared_to_service(service).await?;
        self.install_software_file(service).await?;
        self.install_system_plugin(service).await?;
        self.install_software_lib(service).await?;
        self.set_server_listener(service).await?;
        self.set_plugin_listener(service).await;
        Ok(())
    }

    pub async fn stop_service(&mut self, service_process_ref: &ServiceProcessRef, shutdown_msg: &str) {
        let (id, task_name) = {
            let sp = service_process_ref.read().await;
            (sp.get_id().clone(), sp.get_task_name().to_string())
        };

        match self.task_manager.get_task_ref_from_name(&task_name).await {
            Ok(task_ref) => {
                let (should_delete, timeout) = {
                    let tr = task_ref.read().await;
                    (tr.is_delete(), tr.get_time_shutdown_before_kill())
                };

                {
                    let mut sp = service_process_ref.write().await;
                    sp.shutdown(shutdown_msg, timeout).await;
                }

                if should_delete {
                    service_process_ref.read().await.delete_files();
                    if let Err(e) = TableServices::delete(self.get_db(), &id).await {
                        log_warning!("Error deleting Service {} in DB: {:?}", id, e);
                    }
                    self.services.remove(&id);
                } else {
                    self.update_status(service_process_ref, ServiceStatus::Stopped).await;
                }
            }
            Err(_) => {
                log_info!("Service {} task {} not found — deleting service.", id, task_name);
                let mut sp = service_process_ref.write().await;
                match sp.kill().await {
                    Ok(_) => log_info!("Service killed"),
                    Err(e) => log_warning!("Service cant kill: {}", e),
                }
                sp.delete_files();
                if let Err(e) = TableServices::delete(self.get_db(), &id).await {
                    log_warning!("Error deleting Service {} in DB: {:?}", id, e);
                }
                self.services.remove(&id);
            }
        }
    }

    pub async fn register_on_proxy(&self, service: &Service) -> CloudResult<()> {
        if service.is_proxy() { return Ok(()); }

        for proxy in self.filter_services(|s| s.is_running() && s.is_proxy()).await {
            let s = proxy.read().await;
            let url = s.get_service_url().join("add_server");
            let body = match Utils::convert_to_json(&RegisterServerData {
                register_server: ServiceInfoResponse::new(service),
            }) {
                Some(b) => b,
                None => { log_warning!(2, "Service [{}] can't serialize", service.get_name()); continue; }
            };

            match url.post(&body, Duration::from_secs(3)).await {
                Ok(_) => log_info!(4, "Connected [{}] to Proxy [{}]", service.get_name(), s.get_name()),
                Err(e) => log_warning!(2, "Can't register [{}] to Proxy [{}]: {}", service.get_name(), s.get_name(), e),
            }
        }
        Ok(())
    }

    pub async fn unregister_from_proxy(&self, service: &Service) -> CloudResult<()> {
        if service.is_proxy() { return Ok(()); }

        for proxy in self.filter_services(|s| s.is_proxy() && s.is_start()).await {
            let s = proxy.read().await;
            let url = s.get_service_url().join(format!("remove_server?name={}", service.get_name()).as_str());

            match url.post(&json!({}), Duration::from_secs(3)).await {
                Ok(_) => log_info!(4, "Disconnected [{}] from Proxy [{}]", service.get_name(), s.get_name()),
                Err(e) => log_warning!(2, "Can't unregister [{}] from Proxy [{}]: {}", service.get_name(), s.get_name(), e),
            }
        }
        Ok(())
    }

    pub fn get_from_id(&self, id: &EntityId) -> CloudResult<ServiceProcessRef> {
        self.find_from_id(id).ok_or(error!(CantFindServiceFromUUID))
    }

    pub async fn install_software_file(&self, service: &ServiceProcessRef) -> CloudResult<()> {
        let service_guard = service.read().await;
        let software_link = service_guard.get_config().get_software();

        let (software_path, target_path) = {
            let sc = self.software_config.read().await;
            let software_path = sc.get_software_server_path(software_link);
            let software = sc.get_software(software_link)?;
            let file_name = software.get_software_file().get_file_name();

            (
                software_path.join(&file_name),
                service_guard.get_path().join(&file_name),
            )
        };

        if !software_path.exists() {
            return Err(error!(CantFindSoftwareFile));
        }

        fs::copy(&software_path, &target_path).map_err(|e| error!(CantCopySoftware, e))?;

        Ok(())
    }

    pub async fn install_system_plugin(&self, service: &ServiceProcessRef) -> CloudResult<()> {
        let service_guard = service.read().await;
        let software_link = service_guard.get_config().get_software();

        let sc = self.software_config.read().await;
        let system_plugin_path = sc.get_software_plugin_path(software_link);

        if !system_plugin_path.exists() {
            return Err(error!(CantFindSystemPlugin));
        }

        let software = sc.get_software(software_link)?;
        let plugin = software.get_system_plugin();
        let target_path = service_guard.get_path().join(plugin.get_path());

        fs::create_dir_all(&target_path).map_err(|e| error!(CantCreateSystemPluginPath, e))?;

        match fs::copy(system_plugin_path.join(plugin.get_file_name()), target_path.join(plugin.get_file_name())) {
            Ok(_) => { log_info!("Successfully installed System Plugin"); Ok(()) }
            Err(e) => Err(error!(CantCopySystemPlugin, e)),
        }
    }

    pub async fn install_software_lib(&self, service: &ServiceProcessRef) -> CloudResult<()> {
        let service_guard = service.read().await;
        let sc = self.software_config.read().await;
        let lib_path = sc.get_software_lib_path(service_guard.get_config().get_software());

        Utils::copy_folder_contents(&lib_path, service_guard.get_path(), false)
            .map_err(|e| error!(CantCopySoftwareLib, e))
    }

    pub fn find_from_id(&self, id: &EntityId) -> Option<ServiceProcessRef> {
        self.services.get(id).cloned()
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

    async fn set_server_listener(&self, service: &ServiceProcessRef) -> CloudResult<()> {
        let bind_ports = self.get_bind_ports_except(service).await;

        let mut s = service.write().await;
        let start_port = s.get_config().get_start_port();
        let host = self.config.get_server_host();
        let port = Utils::find_free_port(&bind_ports, start_port, &host);
        let address = Address::new(&host, &port);

        // ip_path + port_path aus SoftwareConfig holen
        let software_link = s.get_config().get_software().clone();
        let software = self.software_config.get_software(&software_link).await?;

        let path = s.get_path().clone();

        let path_ip = path.join(software.get_ip_path());
        if !path_ip.exists() { return Err(error!(CantFindIPConfigFilePath)); }
        let content = read_to_string(&path_ip).map_err(|e| error!(CantReadFileToString, e))?;
        fs::write(&path_ip, content.replace("%ip%", &address.get_ip()))
            .map_err(|e| error!(CantWriteIP, e))?;

        let path_port = path.join(software.get_port_path());
        if !path_port.exists() { return Err(error!(CantFindPortConfigFilePath)); }
        let content = read_to_string(&path_port).map_err(|e| error!(CantReadFileToString, e))?;
        fs::write(&path_port, content.replace("%port%", &address.get_port().to_string()))
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

    pub async fn update_status(&self, service_process_ref: &ServiceProcessRef, status: ServiceStatus) {
        let mut sp = service_process_ref.write().await;
        sp.set_status(status);
        if let Err(e) = TableServices::update(self.get_db(), sp.get_service()).await {
            log_warning!(2, "Cant update Service in DB: {}", e);
        }
    }

    async fn get_bind_ports_except(&self, exclude: &ServiceProcessRef) -> Vec<u32> {
        let host = self.config.get_server_host();
        let mut ports = Vec::new();
        let exclude_id = exclude.get_id().await;

        for (id, arc) in &self.services {
            if *id == exclude_id { continue; }
            let service = arc.read().await;
            let sl = service.get_server_listener();
            if sl.get_ip() == host { ports.push(sl.get_port()); }
            let pl = service.get_plugin_listener();
            if pl.get_ip() == host { ports.push(pl.get_port()); }
        }

        ports
    }

    fn get_db(&self) -> &DatabaseManager { self.db.as_ref() }
}

fn get_all_from_file() -> Vec<ServiceProcessRef> {
    let config = CloudConfig::get();
    let mut list = Vec::new();
    list.extend(get_services_from_path(&config.get_cloud_path().get_service_folder().get_temp_folder_path()));
    list.extend(get_services_from_path(&config.get_cloud_path().get_service_folder().get_static_folder_path()));
    list
}

fn get_services_from_path(path: &PathBuf) -> Vec<ServiceProcessRef> {
    Directory::get_folders_name_from_path(path)
        .into_iter()
        .filter_map(|folder| {
            let p = path.join(&folder);
            get_from_path(&p).map(|service| ServiceProcessRef::new(service, p))
        })
        .collect()
}

fn get_from_path(path: &Path) -> Option<Service> {
    let p = path.join(".minecloud").join("service_config.json");
    read_to_string(p).ok().and_then(|c| serde_json::from_str(&c).ok())
}

impl ServiceManagerRef {
    pub async fn new(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        task_manager: TaskManagerRef,
        software_config: SoftwareConfigRef,
    ) -> CloudResult<Self> {
        let local_services = get_all_from_file();
        TableServices::delete_others(db.as_ref(), &local_services, cloud_config.as_ref()).await?;

        let mut services: HashMap<EntityId, ServiceProcessRef> = HashMap::new();
        for sp_ref in local_services {
            sp_ref.write().await.set_status(ServiceStatus::Stopped);
            TableServices::create_if_not_exists(db.as_ref(), &sp_ref).await?;
            services.insert(sp_ref.get_id().await, sp_ref);
        }

        Ok(ServiceManagerRef(Arc::new(RwLock::new(ServiceManager {
            services, db, config: cloud_config, task_manager, software_config,
        }))))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, ServiceManager> { self.0.read().await }
    pub async fn write(&self) -> RwLockWriteGuard<'_, ServiceManager> { self.0.write().await }

    pub async fn get_service_ref_from_id(&self, id: &EntityId) -> CloudResult<ServiceProcessRef> {
        self.0.read().await.get_from_id(id)
    }
}

impl Clone for ServiceManagerRef {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}