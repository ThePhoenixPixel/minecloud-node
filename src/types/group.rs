use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::types::installer::Installer;
use crate::types::template::Template;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Group {
    name: String,
    installer: Installer,
    templates: Vec<Template>,
}

pub struct GroupRef(Arc<RwLock<Group>>);

impl Group {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_installer(&self) -> &Installer {
        &self.installer
    }

    pub fn get_templates(&self) -> &Vec<Template> {
        &self.templates
    }
}

impl GroupRef {
    pub fn new(group: Group) -> Self {
        Self(Arc::new(RwLock::new(group)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, Group> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, Group> {
        self.0.write().await
    }

    pub fn ptr_eq(&self, other: &GroupRef) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub async fn get_name(&self) -> String {
        self.0.read().await.get_name().to_string()
    }
}

impl Clone for GroupRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


