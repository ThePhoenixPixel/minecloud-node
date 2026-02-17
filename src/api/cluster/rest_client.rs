use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use crate::api::cluster::cluster_client::ClusterClient;
use crate::config::cloud_config::CloudConfig;
use crate::types::Node;
use crate::utils::error::CloudResult;

pub struct RestClusterClient {
    nodes: RwLock<Vec<Node>>,
    cloud_config: Arc<CloudConfig>,
}

impl RestClusterClient {
    pub fn new(cloud_config: Arc<CloudConfig>) -> RestClusterClient {
        RestClusterClient {
            nodes: Vec::new(),
            cloud_config,
        }
    }
}

#[async_trait]
impl ClusterClient for RestClusterClient {
    async fn join_cluster(&self) -> CloudResult<()> {
        todo!()
    }
}

