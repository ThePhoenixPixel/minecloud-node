use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::cluster::{ClusterClient, RestClusterClient};
use crate::config::CloudConfig;
use crate::manager::service_manager::ServiceManager;
use crate::types::{EntityId, Service, ServiceRef, Task};
use crate::utils::error::CloudResult;

pub struct NodeManager {
    service_manager: Arc<RwLock<ServiceManager>>,
    cluster: Box<dyn ClusterClient>,
    cloud_config: Arc<CloudConfig>,
}

impl NodeManager {
    pub async fn new(cloud_config: Arc<CloudConfig>, service_manager: Arc<RwLock<ServiceManager>>) -> CloudResult<NodeManager> {
        Ok(NodeManager {
            service_manager,
            cluster: Box::new(RestClusterClient::new(cloud_config.clone())),
            cloud_config,
        })
    }

    pub async fn stop_all_local_services(&self, msg: &str) {
        let services = self.service_manager.read().await.get_online_all().await;
        for s in services {
            self.stop_service(s.get_id().await, msg).await;
        }
    }

    pub async fn stop_service(&self, id: EntityId, msg: &str) {
        let service_ref = {
            self.service_manager.read().await.find_from_id(&id).await
        };

        if let Some(service_ref) = service_ref {
            // service is local
            match self.unregistered_local_service(&service_ref).await {
                Ok(_) => (),
                Err(e) => (),
            };

            self.service_manager.write().await.stop_service(&id, msg).await;

        } else {
            // service is remote
            todo!("Send stop command to Other Node");
        }
    }
    pub async fn is_responsible_for_task(&self, task: &Task) -> bool {
        task.is_startup_local(&self.cloud_config)
    }

    pub async fn get_all_services_from_task(&self, task_name: &String) -> Vec<Service> {
        let service_refs = self.service_manager.read().await.get_all_from_task(task_name).await;
        let mut services = Vec::new();

        for service_ref in service_refs {
            services.push(service_ref.read().await.get_service().clone());
        }

        services
    }

    pub async fn start_service_from_task(&self, task: &Task) -> CloudResult<()> {
        if self.cloud_config.get_name() != self.find_best_node(task).await {
            // send start request to Node
            return Ok(())
        }
        // start service local
        let service_ref = {
            let mut sm = self.service_manager.write().await;
            sm.get_or_create_service_ref(task).await?
        };
        self.service_manager.read().await.start(service_ref).await?;
        Ok(())
    }

    pub async fn get_online_backend_server(&self) -> Vec<Service> {
        let services = self.service_manager.read().await.get_online_backend_server().await;
        let mut result = Vec::new();
        for s in services {
            result.push(s.read().await.get_service().clone())
        }
        result
    }

    /// Local (Server Plugin called) -> info sent to Cluster
    pub async fn on_local_service_registered(&self, id: EntityId) -> CloudResult<()> {
        let service_ref = {
            self.service_manager.read().await.get_from_id(&id).await?
        };

        self.service_manager.read().await.register_on_proxy(&service_ref.read().await.get_service()).await?;

        Ok(())
    }

    async fn unregistered_local_service(&self, service_ref: &ServiceRef) -> CloudResult<()> {
        self.service_manager.read().await.unregister_from_proxy(&service_ref.read().await.get_service()).await?;

        Ok(())
    }

    /// Remote (Node called) -> info Local
    pub async fn on_remote_service_registered(&self, service: Service) -> CloudResult<()> {
        self.service_manager.read().await.register_on_proxy(&service).await?;
        Ok(())
    }

    /// Local (Server Plugin called) -> info sent to Cluster
    pub async fn on_local_service_shutdown(&self, id: EntityId) -> CloudResult<()> {
        let service_ref = {
            let sm = self.service_manager.read().await;
            sm.get_from_id(&id).await?
        };

        if service_ref.read().await.is_shutdown_init() {
            return Ok(());
        }

        self.unregistered_local_service(&service_ref).await?;

        Ok(())
    }

    /// Remote (Node called) -> info Local
    pub async fn on_remote_service_shutdown(&self, service: Service) -> CloudResult<()> {
        self.service_manager.read().await.unregister_from_proxy(&service).await?;
        Ok(())
    }

    /// find the best Node in Cluster to Start the new Service from Task
    async fn find_best_node(&self, task: &Task) -> String {
        String::from("Node-1")
    }

}


