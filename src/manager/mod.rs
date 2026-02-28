use database_manager::DatabaseManager;
use std::sync::Arc;

pub use node_manager::*;
pub use player_manager::*;
pub use task_manager::*;

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::manager::service_manager::ServiceManagerRef;
use crate::utils::error::CloudResult;

mod node_manager;
mod player_manager;
mod service_manager;
mod task_manager;

pub struct Manager;

impl Manager {
    pub async fn create_all(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        software_config: SoftwareConfigRef,
    ) -> CloudResult<(Arc<PlayerManager>, TaskManagerRef, Arc<NodeManager>)> {
        let task_manager = TaskManagerRef::new(db.clone(), cloud_config.clone(), software_config.clone());
        let service_manager = ServiceManagerRef::new(db.clone(), cloud_config.clone(), task_manager.clone(), software_config.clone()).await?;
        let player_manager = PlayerManager::new(db.clone(), service_manager.clone());
        let node_manager = NodeManager::new(cloud_config.clone(), service_manager, task_manager.clone()).await?;

        Ok((
            Arc::new(player_manager),
            task_manager,
            Arc::new(node_manager),
        ))
    }
}
