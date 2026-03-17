use std::collections::HashMap;
use std::{fs, io};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use database_manager::DatabaseManager;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::config::CloudConfig;
use crate::error;
use crate::types::{Group, GroupRef};
use crate::utils::error::*;

pub struct GroupManager {
    _db: Arc<DatabaseManager>,
    config: Arc<CloudConfig>,

    groups: HashMap<String, GroupRef>,
}

pub struct GroupManagerRef(Arc<RwLock<GroupManager>>);


impl GroupManager {
    pub fn get_all_groups(&self) -> Vec<GroupRef> {
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
            config: cloud_config,
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