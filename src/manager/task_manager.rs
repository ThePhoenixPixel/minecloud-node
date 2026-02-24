use std::collections::HashMap;
use database_manager::DatabaseManager;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use bx::path::Directory;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::error;
use crate::types::{Installer, ServiceRef, Task, TaskRef, Template};
use crate::utils::error::*;


pub struct TaskManager {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,

    tasks: HashMap<String, TaskRef>,
}

pub struct TaskManagerRef(Arc<RwLock<TaskManager>>);


impl TaskManager {
    pub fn get_all_tasks(&self) -> HashMap<String, TaskRef> {
        self.tasks.clone()
    }

    pub fn get_from_name(&self, name: &str) -> CloudResult<TaskRef> {
        for (task_name, task_ref) in &self.tasks {
            if task_name == name {
                return Ok(task_ref.clone());
            }
        }
        Err(error!(CantFindTaskFromName))
    }

    pub async fn get_service_path(&self, task_ref: &TaskRef) -> PathBuf {
        let path = if task_ref.read().await.is_static_service() {
            self.config
                .get_cloud_path()
                .get_service_folder()
                .get_static_folder_path()
        } else {
            self.config
                .get_cloud_path()
                .get_service_folder()
                .get_temp_folder_path()
        };
        path
    }

    #[deprecated]
    pub fn prepared_to_service(&self, service_ref: ServiceRef) -> CloudResult<()> {
        // create the next free service folder with the template
        let target_path = self.create_next_free_service_folder()?;
        for group in self.get_groups() {
            group.install_in_path(&target_path)?;
        }

        let mut templates: Vec<Template> = Vec::new();
        match self.get_installer() {
            Installer::InstallAll => templates = self.get_templates_sorted_by_priority(),
            Installer::InstallAllDesc => templates = self.get_templates_sorted_by_priority_desc(),
            Installer::InstallRandom => match self.get_template_rng() {
                Some(template) => templates.push(template.clone()),
                None => return Err(error!(TemplateNotFound)),
            },
            Installer::InstallRandomWithPriority => {
                match self.get_template_rng_based_on_priority() {
                    Some(template) => templates.push(template.clone()),
                    None => return Err(error!(TemplateNotFound)),
                }
            }
        }

        for template in templates {
            Directory::copy_folder_contents(&template.get_path(), &target_path)
                .map_err(|e| error!(CantCopyTemplateToNewServiceFolder, e))?;
        }
        Ok(target_path)
    }
}

impl TaskManagerRef {
    pub fn new(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        software_config: SoftwareConfigRef,
    ) -> TaskManagerRef {
        let tm = TaskManager {
            db,
            tasks: get_all_task_from_file(&cloud_config),
            config: cloud_config,
            software_config,
        };
        TaskManagerRef(Arc::new(RwLock::new(tm)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, TaskManager> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, TaskManager> {
        self.0.write().await
    }

    pub async fn get_task_ref_from_name(&self, name: &str) -> CloudResult<TaskRef> {
        self.0.read().await.get_from_name(name)
    }

}

impl Clone for TaskManagerRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


fn get_all_task_from_file(config: &Arc<CloudConfig>) -> HashMap<String, TaskRef> {
    let task_path = config.get_cloud_path().get_task_folder_path();

    let mut tasks: HashMap<String, TaskRef> = HashMap::new();

    if task_path.exists() && task_path.is_dir() {
        if let Ok(entries) = fs::read_dir(task_path) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        let name = file_name.trim_end_matches(".json");
                        if let Some(task) = Task::get_task(&name.to_string()) {
                            tasks.insert(task.get_name().to_string(), TaskRef::new(task));
                        }
                    }
                }
            }
        }
    }

    tasks
}

