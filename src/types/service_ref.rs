use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::types::EntityId;
use crate::types::process::ServiceProcess;

pub struct ServiceRef(Arc<RwLock<ServiceProcess>>);

impl ServiceRef {
    pub fn new(process: ServiceProcess) -> Self {
        Self(Arc::new(RwLock::new(process)))
    }

    pub async fn read(&self) -> RwLockReadGuard<'_, ServiceProcess> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, ServiceProcess> {
        self.0.write().await
    }

    pub async fn get_id(&self) -> EntityId {
        self.0.read().await.get_id()
    }

    pub async fn get_name(&self) -> String {
        self.0.read().await.get_name()
    }

    pub async fn is_start(&self) -> bool {
        self.0.read().await.is_start()
    }

    pub async fn is_proxy(&self) -> bool {
        self.0.read().await.is_proxy()
    }
}

impl Clone for ServiceRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
