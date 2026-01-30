use bx::path::Directory;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::service::Service;
use crate::core::task::Task;
use crate::database::database_manger::DatabaseManager;
use crate::sys_config::cloud_config::CloudConfig;
use crate::utils::error::CloudError;

pub struct LocalServices {
    services: Vec<Service>,
    db: Arc<dyn DatabaseManager>
}

impl LocalServices {
    pub fn new(db: Arc<dyn DatabaseManager>) -> LocalServices {
        LocalServices {
            services: LocalServices::get_all_from_file(),
            db,
        }
    }

    pub fn clone(&self) -> LocalServices {
        LocalServices {
            services: self
                .services
                .iter()
                .map(|s| s.clone_without_process())
                .collect(),
            db: self.db.clone(),
        }
    }

    pub fn get_all(&self) -> Vec<Service> {
        self.services
            .iter()
            .map(|s| s.clone_without_process())
            .collect()
    }

    pub fn get_started_services(&self) -> Vec<Service> {
        self.get_all()
            .into_iter()
            .filter(|s| s.is_start())
            .collect()
    }

    pub fn get_prepared_services(&self) -> Vec<Service> {
        self.get_all()
            .into_iter()
            .filter(|s| s.is_prepare())
            .collect()
    }

    pub fn get_stopped_services(&self) -> Vec<Service> {
        self.get_all().into_iter().filter(|s| s.is_stop()).collect()
    }

    pub fn get_start_services_count(&self) -> u32 {
        self.get_started_services().iter().count() as u32
    }

    pub fn get_prepare_services_count(&self) -> u32 {
        self.get_prepared_services().iter().count() as u32
    }

    pub fn get_stop_services_count(&self) -> u32 {
        self.get_stopped_services().iter().count() as u32
    }

    pub fn get_next_stop_service(&self, task: &Task) -> Result<Service, CloudError> {
        if let Some(service) = self
            .services
            .iter()
            .find(|s| s.is_stop() && s.get_task() == *task)
        {
            return Ok(service.clone_without_process());
        }

        self.create(&task)
    }

    pub fn remove_service(&mut self, id: Uuid) -> Option<Service> {
        if let Some(pos) = self.services.iter().position(|s| s.get_id() == id) {
            Some(self.services.remove(pos))
        } else {
            None
        }
    }

    pub fn get_from_id(&self, id: &Uuid) -> Option<Service> {
        self.get_all().into_iter().find(|s| s.get_id() == *id)
    }

    pub fn get_from_id_mut(&mut self, id: &Uuid) -> Option<&mut Service> {
        self.services.iter_mut().find(|s| s.get_id() == *id)
    }

    pub fn get_from_name_mut(&mut self, name: &str) -> Option<&mut Service> {
        self.services.iter_mut().find(|s| s.get_name() == name)
    }

    pub fn set_service(&mut self, mut service: Service) {
        if let Some(existing) = self
            .services
            .iter_mut()
            .find(|s| s.get_id() == service.get_id())
        {
            let s = service.clone_without_process();
            if service.get_process().is_some() {
                existing.set_process(service.extract_process());
            }
            existing.update(&s);
            existing.save_to_file();
        } else {
            (&mut service).new_id();
            service.save_to_file();
            self.services.push(service)
        }
    }

    pub async fn stop_service(&mut self, id: &Uuid, shutdown_msg: &str) {
        if let Some(pos) = self.services.iter().position(|s| s.get_id() == *id) {
            if let Some(service) = self.services.get_mut(pos) {
                service.shutdown(shutdown_msg).await;
                if service.is_delete() {
                    self.services.remove(pos);
                }
            }
        }
    }

    pub async fn stop_all(&mut self, shutdown_msg: &str) {
        let ids: Vec<Uuid> = self.get_all().iter().map(|s| s.get_id()).collect();
        for id in ids {
            self.stop_service(&id, shutdown_msg).await;
        }
    }

    pub fn start_service(&mut self, task: &Task) -> Result<Service, CloudError> {
        let mut service = self.get_next_stop_service(&task)?;
        service = service.start()?;
        Ok(service)
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

    pub fn get_started_proxy_services(&self) -> Vec<Service> {
        self.get_started_services()
            .iter()
            .filter(|s| s.is_proxy())
            .map(|s| s.clone_without_process())
            .collect()
    }

    fn get_start_service_from_file() -> Vec<Service> {
        let mut services = Vec::new();
        for service in LocalServices::get_all_from_file() {
            if service.is_start() {
                services.push(service);
            }
        }
        services
    }

    fn get_prepare_service_from_file() -> Vec<Service> {
        let mut services = Vec::new();
        for service in LocalServices::get_all_from_file() {
            if service.is_prepare() {
                services.push(service);
            }
        }
        services
    }

    pub fn get_bind_ports_from_file() -> Vec<u32> {
        let mut ports: Vec<u32> = Vec::new();
        let mut services: Vec<Service> = LocalServices::get_start_service_from_file();
        services.append(&mut LocalServices::get_prepare_service_from_file());
        for service in services {
            if service.get_server_listener().get_ip() == CloudConfig::get().get_server_host() {
                ports.push(service.get_server_listener().get_port());
            }
            if service.get_plugin_listener().get_ip() == CloudConfig::get().get_server_host() {
                ports.push(service.get_plugin_listener().get_port());
            }
        }
        ports
    }

    pub fn create(&self, task: &Task) -> Result<Service, CloudError> {
        let service = Service::new(task);




        service
    }

    /*
    pub fn get_online_service() -> Vec<Service> {
        let mut service_online_list: Vec<Service> = Vec::new();
        let service_list = LocalServices::get_all_from_file();
        for service in service_list {
            if service.is_start() {
                service_online_list.push(service);
            }
        }
        service_online_list
    }
    pub fn get_prepare_service() -> Vec<Service> {
        let mut service_prepare_list: Vec<Service> = Vec::new();
        let service_list = LocalServices::get_all_from_file();
        for service in service_list {
            if service.is_prepare() {
                service_prepare_list.push(service);
            }
        }
        service_prepare_list
    }

    pub fn get_offline_service() -> Vec<Service> {
        let mut service_offline_list: Vec<Service> = Vec::new();
        let service_list = LocalServices::get_all_from_file();
        for service in service_list {
            if service.is_stop() {
                service_offline_list.push(service);
            }
        }
        service_offline_list
    }
     */
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
