use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::time::Duration;
use bx::path::Path;
use crate::core::service::Service;
use crate::core::task::Task;
use crate::utils::logger::Logger;
use crate::{log_error, log_info};
use crate::sys_config::cloud_config::CloudConfig;

pub struct LocalServices {
    services: Vec<Service>,
}

impl LocalServices {
    pub fn new() -> LocalServices {
        LocalServices {
            services: LocalServices::get_all_from_file(),
        }
    }

    pub fn get_all(&self) -> &Vec<Service> {
        &self.services
    }

    pub fn get_start_services(&self) -> Vec<&Service> {
        let mut services: Vec<&Service> = Vec::new();
        for service in self.get_all() {
            if service.is_start() {
                services.push(service);
            }
        }
        services
    }

    pub fn get_prepared_services(&self) -> Vec<&Service> {
        let mut services: Vec<&Service> = Vec::new();
        for service in self.get_all() {
            if service.is_prepare() {
                services.push(service);
            }
        }
        services
    }

    pub fn get_stop_services(&self) -> Vec<&Service> {
        let mut services: Vec<&Service> = Vec::new();
        for service in self.get_all() {
            if service.is_stop() {
                services.push(service);
            }
        }
        services
    }

    pub fn get_start_services_count(&self) -> u32 {
        self.get_all()
            .iter()
            .filter(|service| service.is_start())
            .count() as u32
    }

    pub fn get_prepare_services_count(&self) -> u32 {
        self.get_all()
            .iter()
            .filter(|service| service.is_prepare())
            .count() as u32
    }

    pub fn get_stop_services_count(&self) -> u32 {
        self.get_all()
            .iter()
            .filter(|service| service.is_stop())
            .count() as u32
    }

    pub fn get_next_stop_service(&self, task: &Task) -> Result<Service, Error> {
        let offline_services = self.get_stop_services();
        for offline_service in offline_services {
            if !(offline_service.get_task() == task.clone()) {
                continue;
            }
            return Ok(offline_service.clone_without_process());
        }

        match Service::new_local(&task) {
            Ok(service) => Ok(service),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }

    pub fn get_from_name_mut(&mut self, name: &str) -> Option<&mut Service> {
        self.services.iter_mut().find(|s| s.get_name() == name)
    }

    pub fn add_service(&mut self, service: Service) {
        self.services.push(service)
    }

    pub async fn stop_service(&mut self, service_name: &str, shutdown_msg: &str) {
        if let Some(pos) = self
            .services
            .iter()
            .position(|s| s.get_name() == service_name)
        {
            if let Some(service) = self.services.get_mut(pos) {
                service.shutdown(shutdown_msg).await;
            }
            self.services.remove(pos);
        }
    }

    pub async fn stop_all(&mut self, shutdown_msg: &str) {
        let names: Vec<String> = self
            .services
            .iter()
            .map(|s| s.get_name().to_string())
            .collect();
        for name in names {
            self.stop_service(&name, shutdown_msg).await;
        }
    }

    pub async fn check_service(&mut self, task: Task) {
        log_info!(
            "(Local) Task: {} | Start Service: {} | Prepare Service: {} | Stop Service: {}",
            task.get_name(),
            self.get_start_services_count(),
            self.get_prepare_services_count(),
            self.get_stop_services_count()
        );

        let service_count =
            task.get_min_service_count() as u64 - Service::get_starts_service_from_task(&task);

        for _ in 0..service_count {

            log_info!("Service would be created from task: {}", task.get_name());
            log_info!("---------------------------------------------------------------");

            let service = match self.get_next_stop_service(&task) {
                Ok(service) => service,
                Err(e) => {
                    log_error!("{}", e);
                    return;
                }
            };

            let service_name = service.get_name();

            match self.start_service(&task).await {
                Ok(_) => log_info!("Server [{}] successfully start :=)", service_name),
                Err(e) => log_error!("Server [{}] can NOT Start \n {}", service_name, e),
            }


            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn start_service(&mut self, task: &Task) -> Result<(), Error> {
        let service = self.get_next_stop_service(&task)?;
        let service = service.start().await?;
        self.add_service(service);
        Ok(())
    }

    pub fn get_all_from_file() -> Vec<Service> {
        let mut service_list: Vec<Service> = Vec::new();
        service_list.append(
            &mut get_services_from_path(
                &CloudConfig::get()
                    .get_cloud_path()
                    .get_service_folder()
                    .get_temp_folder_path()
            )
        );

        service_list.append(
            &mut get_services_from_path(
                &CloudConfig::get()
                    .get_cloud_path()
                    .get_service_folder()
                    .get_static_folder_path()
            )
        );

        service_list
    }

    pub fn get_start_proxy_servers(&self) -> Vec<&Service> {
        let services = self.get_start_services();
        let mut proxy_server_list: Vec<&Service> = Vec::new();

        for service in services {
            if service.is_proxy() {
                proxy_server_list.push(service)
            }
        }
        proxy_server_list
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
        let mut services : Vec<Service> = LocalServices::get_start_service_from_file();
        services.append(&mut LocalServices::get_prepare_service_from_file());
        for service in services {
            if service.get_server_address().get_ip() == CloudConfig::get().get_server_host() {
                ports.push(service.get_server_address().get_port());
            }
            if service.get_plugin_listener().get_ip() == CloudConfig::get().get_server_host() {
                ports.push(service.get_plugin_listener().get_port());
            }
        }
        ports
    }

    pub fn update_service(&mut self, s: Service) {
        for service in &mut self.services {
            if service.get_id() == s.get_id() {
                service.update(&s);
                service.save_to_file();
                return;
            }
        }
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
    for folder in Path::get_folders_name_from_path(&path) {
        let mut path = path.clone();
        path.push(folder);
        if let Some(service) = Service::get_from_path(&mut path) {
            service_list.push(service);
        };
    }
    service_list
}


