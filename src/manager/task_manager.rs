use database_manager::DatabaseManager;
use std::fs;
use std::sync::Arc;

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::error;
use crate::types::{Task, TaskRef};
use crate::utils::error::*;

pub struct TaskManager {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,

    tasks: Vec<TaskRef>,
}

impl TaskManager {
    pub fn new(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        software_config: SoftwareConfigRef,
    ) -> TaskManager {
        TaskManager {
            db,
            config: cloud_config,
            software_config,
            tasks: Self::get_all_task_from_file(),
        }
    }

    pub fn get_all_task(&self) -> Vec<TaskRef> {
        self.tasks.clone()
    }

    pub async fn get_from_name(&self, name: &String) -> CloudResult<TaskRef> {
        for arc in &self.tasks {
            if arc.get_name().await == *name {
                return Ok(arc.clone());
            }
        }
        Err(error!(CantFindTaskFromName))
    }

    pub fn get_all_task_from_file() -> Vec<TaskRef> {
        let task_path = CloudConfig::get().get_cloud_path().get_task_folder_path();

        let mut tasks: Vec<TaskRef> = Vec::new();

        if task_path.exists() && task_path.is_dir() {
            if let Ok(entries) = fs::read_dir(task_path) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".json") {
                            let name = file_name.trim_end_matches(".json");
                            if let Some(task) = Task::get_task(&name.to_string()) {
                                tasks.push(TaskRef::new(task));
                            }
                        }
                    }
                }
            }
        }

        tasks
    }
}
