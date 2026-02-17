use bx::path::Directory;
use std::path::PathBuf;
use uuid::Uuid;

use crate::types::service::Service;
use crate::types::task::Task;
use crate::database::manager::DatabaseManager;
use crate::config::cloud_config::CloudConfig;
use crate::utils::error::cloud_error::CloudError;
use crate::utils::error::error_kind::CloudErrorKind::*;

pub struct LocalServices {
    services: Vec<Service>,
    db: DatabaseManager
}

impl LocalServices {
    pub fn new(db: DatabaseManager) -> LocalServices {
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
            .filter(|s| s.is-pre())
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



    pub async fn stop_service(&mut self, id: &Uuid, shutdown_msg: &str) {
        if let Some(pos) = self.services.iter().position(|s| s.get_id() == *id) {
            if let Some(service) = self.services.get_mut(pos) {
                //service.shutdown(shutdown_msg).await;
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
}


