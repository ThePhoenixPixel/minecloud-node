use std::io::Error;
use uuid::Uuid;

use crate::core::service::Service;
use crate::core::services_local::LocalServices;
use crate::core::services_network::NetworkServices;
use crate::core::task::Task;
use crate::utils::logger::Logger;
use crate::{log_error, log_info};

pub struct AllServices {
    local_services: LocalServices,
    network_services: NetworkServices,
}

impl AllServices {
    pub fn new(local_services: LocalServices, network_services: NetworkServices) -> Self {
        Self {
            local_services,
            network_services,
        }
    }

    pub fn clone(&self) -> AllServices {
        AllServices {
            local_services: self.local_services.clone(),
            network_services: self.network_services.clone(),
        }
    }

    pub async fn get_all(&self) -> Vec<Service> {
        let mut result: Vec<Service> = Vec::new();
        let local_services: Vec<Service> = self
            .local_services
            .get_all()
            .into_iter()
            .map(|s| s.clone_without_process())
            .collect();

        result.extend(local_services);

        // Netzwerk-Services (nehmen wir an: liefert Vec<Service>)
        result.extend(self.network_services.get_all().await);

        result
    }

    pub fn get_local(&self) -> &LocalServices {
        &self.local_services
    }

    pub fn get_local_mut(&mut self) -> &mut LocalServices {
        &mut self.local_services
    }

    pub fn get_network(&self) -> &NetworkServices {
        &self.network_services
    }

    pub fn get_network_mut(&mut self) -> &mut NetworkServices {
        &mut self.network_services
    }

    pub async fn get_start_services(&self) -> Vec<Service> {
        // lokale starten
        let result: Vec<Service> = self
            .local_services
            .get_started_services()
            .into_iter()
            .map(|s| s.clone_without_process())
            .collect();

        // TODO: Netzwerk-Logik ergänzen
        result
    }

    pub async fn get_prepare_services(&self) -> Vec<Service> {
        let result: Vec<Service> = self
            .local_services
            .get_prepared_services()
            .into_iter()
            .map(|s| s.clone_without_process())
            .collect();

        // TODO: Netzwerk-Logik ergänzen
        result
    }

    pub async fn get_stopped_services(&self) -> Vec<Service> {
        let result: Vec<Service> = self
            .local_services
            .get_stopped_services()
            .into_iter()
            .map(|s| s.clone_without_process())
            .collect();

        // TODO: Netzwerk-Logik ergänzen
        result
    }

    pub async fn get_online_backend_services(&self) -> Vec<Service> {
        self.get_all()
            .await
            .into_iter()
            .filter(|s| s.is_backend_server() && s.is_start())
            .collect()
    }

    pub async fn get_online_proxy_services(&self) -> Vec<Service> {
        self.get_all()
            .await
            .into_iter()
            .filter(|s| s.is_proxy() && s.is_start())
            .collect()
    }

    pub async fn get_from_id(&self, id: &Uuid) -> Option<Service> {
        self.get_all()
            .await
            .into_iter()
            .find(|s| s.get_id() == *id)
            .map(|s| s.clone_without_process())
    }

    pub async fn check_service(&mut self) {
        for task in Task::get_task_all() {
            if task.is_startup_local() {
                let local = self.get_local_mut();

                log_info!(
                    "(Local) Task: {} | Start Service: {} | Prepare Service: {} | Stop Service: {}",
                    task.get_name(),
                    local.get_start_services_count(),
                    local.get_prepare_services_count(),
                    local.get_stop_services_count()
                );

                let service_count = task.get_min_service_count() as u64
                    - Service::get_starts_service_from_task(&task);

                for _ in 0..service_count {
                    log_info!("Service would be created from task: {}", task.get_name());
                    log_info!("---------------------------------------------------------------");

                    match local.start_service(&task) {
                        Ok(service) => {
                            log_info!("Server [{}] successfully start :=)", service.get_name());
                            local.set_service(service); // Child speichern
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                        Err(e) => {
                            log_error!(
                                "Server From Task [{}] can NOT Start \n {}",
                                task.get_name(),
                                e
                            );
                            continue;
                        }
                    }
                }
            } else {
                // TODO: Netzwerk starten
            }
        }
    }

    pub async fn start_service(&mut self, task: &Task) -> Result<(), Error> {
        if task.is_startup_local() {
            let s = self.get_local_mut().start_service(&task)?;
            self.local_services.set_service(s);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else {
            // TODO: Netzwerk-Startlogik
        }
        Ok(())
    }
}
