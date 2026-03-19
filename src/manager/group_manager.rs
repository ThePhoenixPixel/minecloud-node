use std::collections::HashMap;
use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use bx::path::Directory;
use database_manager::DatabaseManager;
use rand::prelude::IteratorRandom;
use rand::RngExt;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::config::CloudConfig;
use crate::error;
use crate::types::{Group, GroupRef, Installer, Template};
use crate::utils::error::*;

pub struct GroupManager {
    _db: Arc<DatabaseManager>,
    _config: Arc<CloudConfig>,

    groups: HashMap<String, GroupRef>,
}

pub struct GroupManagerRef(Arc<RwLock<GroupManager>>);


impl GroupManager {
    pub fn get_all(&self) -> Vec<GroupRef> {
        let mut groups: Vec<GroupRef> = Vec::new();
        for (_, group) in &self.groups {
            groups.push(group.clone())
        }

        groups
    }

    pub fn get_from_name(&self, name: &str) -> CloudResult<GroupRef> {
        for (group_name, group_ref) in &self.groups {
            if group_name == name {
                return Ok(group_ref.clone());
            }
        }
        Err(error!(CantFindGroupFromName))
    }

    pub async fn filter_groups<F>(&self, mut filter: F) -> Vec<GroupRef>
    where
        F: FnMut(&Group) -> bool,
    {
        let mut result = Vec::new();

        for arc in self.groups.values() {
            let sp = arc.read().await;

            if filter(&sp) {
                result.push(arc.clone());
            }
        }

        result
    }

    pub async fn get_templates_sorted_by_priority(&self, group_ref: &GroupRef) -> Vec<Template> {
        let mut templates = group_ref.read().await.get_templates().clone();
        templates.sort_by(|a, b| a.priority.cmp(&b.priority));
        templates
    }

    pub async fn get_templates_sorted_by_priority_desc(&self, group_ref: &GroupRef) -> Vec<Template> {
        let mut templates = group_ref.read().await.get_templates().clone();
        templates.sort_by(|a, b| b.priority.cmp(&a.priority));
        templates
    }

    pub async fn get_template_rng(&self, group_ref: &GroupRef) -> Option<Template> {
        let mut rng = rand::rng();
        group_ref.read().await.get_templates().clone().into_iter().choose(&mut rng)
    }

    // Select Template based on Priority (higher priority = higher chance)
    pub async fn get_template_rng_based_on_priority(&self, group_ref: &GroupRef) -> Option<Template> {
        let templates = {
            group_ref.read().await.get_templates().clone()
        };

        if templates.is_empty() {
            return None;
        }

        let total_weight: u32 = templates.iter().map(|t| t.priority).sum();

        if total_weight == 0 {
            return self.get_template_rng(group_ref).await;
        }

        let mut rng = rand::rng();
        let mut random_value = rng.random_range(0..total_weight);

        for template in &templates {
            if random_value < template.get_priority() {
                return Some(template.clone());
            }
            random_value -= template.priority;
        }

        // fallback
        templates.last().cloned()
    }

    pub async fn install_in_path(&self, group_ref: &GroupRef, target_path: &PathBuf) -> CloudResult<()> {
        let mut templates: Vec<Template> = Vec::new();
        let group = {
            let g_ref = group_ref.read().await;
            g_ref.clone()
        };

        match group.get_installer() {
            Installer::InstallAll => templates = self.get_templates_sorted_by_priority(group_ref).await,
            Installer::InstallAllDesc => templates = self.get_templates_sorted_by_priority_desc(group_ref).await,
            Installer::InstallRandom => match self.get_template_rng(group_ref).await {
                Some(template) => templates.push(template),
                None => return Err(error!(GroupTemplateNotFound)),
            },
            Installer::InstallRandomWithPriority => {
                match self.get_template_rng_based_on_priority(group_ref).await {
                    Some(template) => templates.push(template.clone()),
                    None => return Err(error!(GroupTemplateNotFound)),
                }
            }
        }

        for template in templates {
            Directory::copy_folder_contents(&template.get_path(), target_path)
                .map_err(|e| error!(CantCopyGroupTemplateToNewServiceFolder, e))?;
        }
        Ok(())
    }

}

impl GroupManagerRef {
    pub fn new(
        _db: Arc<DatabaseManager>,
        cloud_config: Arc<CloudConfig>,
    ) -> GroupManagerRef {
        let groups = get_all_groups_from_file(&cloud_config);
        let gm = GroupManager {
            _db,
            groups,
            _config: cloud_config,
        };

        GroupManagerRef(Arc::new(RwLock::new(gm)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, GroupManager> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, GroupManager> {
        self.0.write().await
    }

    pub async fn get_group_ref_from_name(&self, name: &str) -> CloudResult<GroupRef> {
        self.0.read().await.get_from_name(name)
    }

}

impl Clone for GroupManagerRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


fn get_all_groups_from_file(config: &Arc<CloudConfig>) -> HashMap<String, GroupRef> {
    let group_path = config.get_cloud_path().get_group_folder_path();
    let mut groups: HashMap<String, GroupRef> = HashMap::new();

    if !group_path.exists() || !group_path.is_dir() {
        return groups;
    }

    if let Ok(entries) = fs::read_dir(group_path) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    let group_name = file_name.trim_end_matches(".json").to_string();
                    if let Ok(group) = from_path(&entry.path()) {
                        groups.insert(group_name, GroupRef::new(group));
                    }
                }
            }
        }
    }

    groups
}

fn from_path(path: &PathBuf) -> io::Result<Group> {
    let mut file = File::open(path)?;
    let mut content = String::new();

    file.read_to_string(&mut content)?;

    let group: Group = serde_json::from_str(&content)?;

    Ok(group)
}