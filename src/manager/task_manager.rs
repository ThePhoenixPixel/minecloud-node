use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use database_manager::DatabaseManager;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use bx::path::Directory;

use crate::config::{CloudConfig, SoftwareConfigRef};
use crate::manager::GroupManagerRef;
use crate::types::{Installer, ServiceProcessRef, SoftwareLink, Task, TaskRef, Template};
use crate::utils::error::*;
use crate::{error, log_info, log_warning};

pub struct TaskManager {
    db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,
    software_config: SoftwareConfigRef,
    group_manager: GroupManagerRef,
    tasks: HashMap<String, TaskRef>,
}

pub struct TaskManagerRef(Arc<RwLock<TaskManager>>);

impl TaskManager {
    pub async fn create_task(&mut self, name: String, software_link: SoftwareLink) -> CloudResult<TaskRef> {
        if self.is_task_exists(&name) {
            return Err(error!(TaskAlreadyExists));
        }

        let software = self.software_config.get_software(&software_link).await?;
        let task = Task::new(name.clone(), software_link, software.get_max_ram());
        let task_ref = TaskRef::new(task.clone());

        self.save_task(&task)?;
        self.tasks.insert(name, task_ref.clone());

        Ok(task_ref)
    }

    pub async fn update_task(&mut self, name: &str, new_task: Task) -> CloudResult<()> {
        // Name geändert → alte Datei löschen
        if name != new_task.get_name() {
            self.delete_task_file(name);
            self.tasks.remove(name);
        }

        self.save_task(&new_task)?;

        let task_ref = self.tasks
            .entry(new_task.get_name())
            .or_insert_with(|| TaskRef::new(new_task.clone()));

        *task_ref.write().await = new_task;

        Ok(())
    }

    pub fn delete(&mut self, name: &str) {
        self.delete_task_file(name);
        self.tasks.remove(name);
        log_info!(6, "Task |{}| successfully removed", name);
    }

    pub fn save_task(&self, task: &Task) -> CloudResult<()> {
        let path = self.get_task_path(&task.get_name());

        // Template-Ordner anlegen falls neue Task
        if !path.exists() {
            Template::create_by_task(task);
        }

        let serialized = serde_json::to_string_pretty(task)
            .map_err(|e| error!(CantSerializeTask, e))?;

        let mut file = File::create(&path)
            .map_err(|e| error!(CantCreateTaskFile, e))?;

        file.write_all(serialized.as_bytes())
            .map_err(|e| error!(CantWriteTaskFile, e))?;

        Ok(())
    }

    pub fn is_task_exists(&self, name: &str) -> bool {
        self.tasks.contains_key(name)
    }

    pub fn get_all_tasks(&self) -> Vec<TaskRef> {
        self.tasks.values().cloned().collect()
    }

    pub fn get_from_name(&self, name: &str) -> CloudResult<TaskRef> {
        self.tasks.get(name).cloned().ok_or(error!(CantFindTaskFromName))
    }

    pub async fn filter_tasks<F>(&self, mut filter: F) -> Vec<TaskRef>
    where
        F: FnMut(&Task) -> bool,
    {
        let mut result = Vec::new();
        for arc in self.tasks.values() {
            let sp = arc.read().await;
            if filter(&sp) {
                result.push(arc.clone());
            }
        }
        result
    }

    pub async fn get_service_path(&self, task_ref: &TaskRef) -> PathBuf {
        if task_ref.read().await.is_static_service() {
            self.config.get_cloud_path().get_service_folder().get_static_folder_path()
        } else {
            self.config.get_cloud_path().get_service_folder().get_temp_folder_path()
        }
    }

    pub async fn prepared_to_service(&self, service_ref: &ServiceProcessRef) -> CloudResult<()> {
        let task = {
            let s_ref = service_ref.read().await;
            let tasks = self.filter_tasks(|t| t.get_name() == s_ref.get_task_name()).await;
            match tasks.first() {
                Some(t) => t.read().await.clone(),
                None => return Err(error!(CantFindTaskFromName)),
            }
        };

        let target_path = {
            let s_ref = service_ref.read().await;
            let path = s_ref.get_path().clone();
            fs::create_dir_all(&path).into_cloud_error(CantCreateServiceFolder)?;
            path
        };

        // Groups installieren
        {
            let gm = self.group_manager.read().await;
            for group_name in task.get_group_names() {
                let group_ref = match gm.get_from_name(group_name) {
                    Ok(g) => g,
                    Err(e) => {
                        log_warning!(3, "Group |{}| not found in Task |{}|: {}", group_name, task.get_name(), e);
                        continue;
                    }
                };
                match gm.install_in_path(&group_ref, &target_path).await {
                    Ok(_) => log_info!(7, "Group |{}| installed in |{:?}|", group_name, target_path),
                    Err(e) => log_warning!(3, "Group |{}| cant install: {}", group_name, e),
                }
            }
        }

        // Templates kopieren
        for template in get_templates_by_installer(&task)? {
            Directory::copy_folder_contents(&template.get_path(), &target_path)
                .map_err(|e| error!(CantCopyTemplateToNewServiceFolder, e))?;
        }

        Ok(())
    }

    fn get_task_path(&self, name: &str) -> PathBuf {
        self.config.get_cloud_path().get_task_folder_path().join(format!("{}.json", name))
    }

    fn delete_task_file(&self, name: &str) {
        let path = self.get_task_path(name);
        match fs::remove_file(&path) {
            Ok(_) => log_info!(6, "Task file |{}| deleted", name),
            Err(e) => log_warning!("Task file |{}| cant delete: {}", name, e),
        }
    }
}

impl TaskManagerRef {
    pub fn new(
        db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
        software_config: SoftwareConfigRef,
        group_manager: GroupManagerRef,
    ) -> TaskManagerRef {
        let tasks = load_tasks_from_file(&cloud_config);
        let tm = TaskManager { db, tasks, config: cloud_config, software_config, group_manager };
        TaskManagerRef(Arc::new(RwLock::new(tm)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, TaskManager> { self.0.read().await }
    pub async fn write(&self) -> RwLockWriteGuard<'_, TaskManager> { self.0.write().await }

    pub async fn get_task_ref_from_name(&self, name: &str) -> CloudResult<TaskRef> {
        self.0.read().await.get_from_name(name)
    }
}

impl Clone for TaskManagerRef {
    fn clone(&self) -> Self { Self(self.0.clone()) }
}

// --- private Hilfsfunktionen ---

fn load_tasks_from_file(config: &Arc<CloudConfig>) -> HashMap<String, TaskRef> {
    let task_path = config.get_cloud_path().get_task_folder_path();
    let mut tasks = HashMap::new();

    if !task_path.exists() { return tasks; }

    let entries = match fs::read_dir(&task_path) {
        Ok(e) => e,
        Err(e) => { log_warning!("Cant read task folder: {}", e); return tasks; }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => { log_warning!("Cant read task file {:?}: {}", path, e); continue; }
        };

        match serde_json::from_str::<Task>(&content) {
            Ok(task) => { tasks.insert(task.get_name(), TaskRef::new(task)); }
            Err(e) => log_warning!("Cant parse task file {:?}: {}", path, e),
        }
    }

    tasks
}

fn get_templates_by_installer(task: &Task) -> CloudResult<Vec<Template>> {
    let templates = match task.get_installer() {
        Installer::InstallAll => task.get_templates_sorted_by_priority(),
        Installer::InstallAllDesc => task.get_templates_sorted_by_priority_desc(),
        Installer::InstallRandom => match task.get_template_rng() {
            Some(t) => vec![t.clone()],
            None => return Err(error!(TemplateNotFound)),
        },
        Installer::InstallRandomWithPriority => match task.get_template_rng_based_on_priority() {
            Some(t) => vec![t.clone()],
            None => return Err(error!(TemplateNotFound)),
        },
    };
    Ok(templates)
}