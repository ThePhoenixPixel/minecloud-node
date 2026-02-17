use std::sync::Arc;
use database_manager::DatabaseManager;

use crate::api::cluster::{ClusterClient, RestClusterClient};
use crate::config::CloudConfig;
use crate::manager::service_manager::ServiceManager;
use crate::utils::error::CloudResult;

pub struct NodeManager {
    service_manager: Arc<ServiceManager>,
    cluster: Box<dyn ClusterClient>,
    cloud_config: Arc<CloudConfig>,
}

impl NodeManager {
    pub async fn new(cloud_config: Arc<CloudConfig>, service_manager: Arc<ServiceManager>) -> CloudResult<NodeManager> {
        Ok(NodeManager {
            service_manager,
            cluster: Box::new(RestClusterClient::new(cloud_config.clone())),
            cloud_config,
        })
    }

    pub async fn stop_all_local_services(&self, msg: &str) {
        todo!("Stop all local servcies");
    }

}


