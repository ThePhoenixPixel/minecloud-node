use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::cloud_config::CloudConfig;
use crate::config::software_config::SoftwareConfig;
use crate::database::manager::DatabaseManager;
use crate::types::task::Task;

pub struct TaskManager {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: Arc<RwLock<SoftwareConfig>>,
    
    tasks: Vec<Task>,
    
}


impl TaskManager {
    pub fn new(db: Arc<DatabaseManager>, cloud_config: Arc<CloudConfig>, software_config: Arc<RwLock<SoftwareConfig>>) -> TaskManager {
        TaskManager {
            db,
            config: cloud_config,
            software_config,
            tasks: Task::get_task_all(),
        }
    }
    
    pub fn get_all_task(&self) -> Vec<Task> {
        self.tasks.clone()
    }
    
}
