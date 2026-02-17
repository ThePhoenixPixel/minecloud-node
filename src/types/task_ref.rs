use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use crate::config::CloudConfig;
use crate::types::Task;


pub struct TaskRef(Arc<RwLock<Task>>);

impl TaskRef {
    pub fn new(task: Task) -> Self {
        Self(Arc::new(RwLock::new(task)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, Task> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, Task> {
        self.0.write().await
    }

    pub fn ptr_eq(&self, other: &TaskRef) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub async fn get_name(&self) -> String {
        self.0.read().await.get_name()
    }

    pub async fn is_startup_local(&self, config: &Arc<CloudConfig>) -> bool {
        self.0.read().await.is_startup_local(config)
    }
}

impl Clone for TaskRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

