use std::sync::Arc;
use database_manager::DatabaseManager;
use tokio::sync::RwLock;

pub use player_manager::PlayerManager;
pub use node_manager::NodeManager;
pub use task_manager::TaskManager;

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::manager::service_manager::ServiceManager;
use crate::utils::error::CloudResult;

mod service_manager;
mod task_manager;
mod player_manager;
mod node_manager;

pub struct Manager;

impl Manager {
    pub async fn create_all(db: Arc<DatabaseManager>, cloud_config: Arc<CloudConfig>, software_config: SoftwareConfigRef) -> CloudResult<(Arc<PlayerManager>, Arc<TaskManager>, Arc<NodeManager>)> {
        let service_manager = Arc::new(RwLock::new(ServiceManager::new(db.clone(), cloud_config.clone(), software_config.clone()).await?));
        let player_manager = PlayerManager::new(db.clone(), service_manager.clone());
        let node_manager = NodeManager::new(cloud_config.clone(), service_manager).await?;
        let task_manager = TaskManager::new(db.clone(), cloud_config.clone(), software_config.clone());

        Ok((Arc::new(player_manager), Arc::new(task_manager), Arc::new(node_manager)))
    }
}




