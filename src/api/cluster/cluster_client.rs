use async_trait::async_trait;

use crate::utils::error::CloudResult;

#[async_trait]
pub trait ClusterClient: Send + Sync {
    async fn join_cluster(&self) -> CloudResult<()>;
}
