use std::io::Error;

use crate::core::service::Service;
use crate::core::services_local::LocalServices;
use crate::core::services_network::NetworkServices;
use crate::core::task::Task;

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

    pub async fn get_all(&self) -> Vec<&Service> {
        let mut result = Vec::new();
        result.extend(self.local_services.get_all());
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
    
    pub async fn get_start_services(&self) -> Vec<&Service> {
        self.get_local().get_start_services()
        // netzwerk logic
    }
    
    pub async fn get_prepare_services(&self) -> Vec<&Service> {
        self.get_local().get_prepared_services()
        // netzwerk logic
    }
    
    pub async fn get_stop_services(&self) -> Vec<&Service> {
        self.get_local().get_stop_services()
        // netzwerklogic
    }
    
    pub async fn get_online_backend_services(&self) -> Vec<&Service> {
        self.get_all()
            .await
            .into_iter()
            .filter(|s| s.is_backend_server() && s.is_start())
            .collect()
    }

    pub async fn get_online_proxy_services(&self) -> Vec<&Service> {
        self.get_all()
            .await
            .into_iter()
            .filter(|s| s.is_proxy() && s.is_start())
            .collect()
    }

    pub async fn check_service(&mut self) {
        for task in Task::get_task_all() {
            if task.is_startup_local() {
                self.get_local_mut().check_service(task).await
            }
        }
    }

    pub async fn start_service(&mut self, task: Task) -> Result<(), Error>{
        if task.is_startup_local() {
            self.get_local_mut().start_service(&task).await?;
        } else {
            todo!()
        }
        Ok(())
    }
}
