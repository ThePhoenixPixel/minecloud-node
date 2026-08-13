use database_manager::DatabaseManager;
use std::sync::Arc;

pub use group_manager::*;
pub use node_manager::*;
pub use player_manager::*;
pub use task_manager::*;

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::manager::service_manager::ServiceManagerRef;
use crate::utils::error::CloudResult;

mod group_manager;
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
    ) -> CloudResult<(
        PlayerManagerRef,
        TaskManagerRef,
        Arc<NodeManager>,
        GroupManagerRef,
    )> {
        let group_manager = GroupManagerRef::new(db.clone(), cloud_config.clone());
        let task_manager = TaskManagerRef::new(
            db.clone(),
            cloud_config.clone(),
            software_config.clone(),
            group_manager.clone(),
        );
        let service_manager = ServiceManagerRef::new(
            db.clone(),
            cloud_config.clone(),
            task_manager.clone(),
            software_config.clone(),
        )
        .await?;
        let player_manager = PlayerManagerRef::new(db.clone(), service_manager.clone(), task_manager.clone()).await;

        let node_manager =
            NodeManager::new(cloud_config.clone(), service_manager, task_manager.clone()).await?;

        Ok((
            player_manager,
            task_manager,
            Arc::new(node_manager),
            group_manager,
        ))
    }
}
