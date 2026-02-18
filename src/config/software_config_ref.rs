use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::{CloudConfig, SoftwareConfig};


pub struct SoftwareConfigRef(Arc<RwLock<SoftwareConfig>>);

impl SoftwareConfigRef {
    pub fn new(cloud_config: Arc<CloudConfig>) -> SoftwareConfigRef {
        SoftwareConfigRef(Arc::new(RwLock::new(SoftwareConfig::new(cloud_config))))
    }
}

impl Clone for SoftwareConfigRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

