use std::sync::Arc;
use database_manager::DatabaseManager;

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::types::Task;

pub struct TaskManager {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,
    
    tasks: Vec<Task>,
    
}


impl TaskManager {
    pub fn new(db: Arc<DatabaseManager>, cloud_config: Arc<CloudConfig>, software_config: SoftwareConfigRef) -> TaskManager {
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
