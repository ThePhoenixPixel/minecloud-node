use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::cluster::{ClusterClient, RestClusterClient};
use crate::config::CloudConfig;
use crate::manager::service_manager::ServiceManager;
use crate::types::{Service, Task};
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
        todo!("Stop all local servcies");
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
        if !(self.cloud_config.get_name() == self.find_best_node(task).await) {
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

    /// find the best Node in Cluster to Start the new Service from Task
    async fn find_best_node(&self, task: &Task) -> String {
        String::from("Node-1")
    }

}


