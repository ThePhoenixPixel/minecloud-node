use std::sync::Arc;
use tokio::sync::RwLock;
use database_manager::DatabaseManager;

use crate::api::cluster::{ClusterClient, RestClusterClient};
use crate::config::cloud_config::CloudConfig;
use crate::config::software_config::SoftwareConfig;
use crate::manager::service_manager::ServiceManager;
use crate::utils::error::CloudResult;

pub struct NodeManager {
    service_manager: ServiceManager,
    cluster: Box<dyn ClusterClient>,
    cloud_config: Arc<CloudConfig>,
}

impl NodeManager {
    pub async fn new(db: Arc<DatabaseManager>, cloud_config: Arc<CloudConfig>, software_config: Arc<RwLock<SoftwareConfig>>) -> CloudResult<NodeManager> {
        Ok(NodeManager {
            service_manager: ServiceManager::new(db, cloud_config.clone(), software_config).await?,
            cluster: Box::new(RestClusterClient::new(cloud_config.clone())),
            cloud_config,
        })
    }



}


