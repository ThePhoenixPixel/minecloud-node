use database_manager::DatabaseManager;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use bx::path::Directory;
use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::error;
use crate::types::{Installer, ServiceRef, Task, TaskRef, Template};
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

    pub fn get_service_path(&self, task: &Task) -> PathBuf {
        let path = if task.is_static_service() {
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
