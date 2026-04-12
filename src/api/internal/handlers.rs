use std::sync::Arc;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::api::internal::{PlayerActionRequest, ServiceIdRequest, ServiceInfoResponse};
use crate::cloud::Cloud;
use crate::types::EntityId;
use crate::utils::error::CloudResult;
use crate::utils::utils::Utils;

pub struct APIInternalHandler;

impl APIInternalHandler {
    /// Called by the Minecraft Process (Minecraft Plugin) when a service is shutdown
    pub async fn service_notify_shutdown(
        cloud: Arc<RwLock<Cloud>>,
        request: ServiceIdRequest,
    ) -> CloudResult<()> {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        node_manager
            .on_local_service_shutdown(EntityId::from(&request))
            .await?;

        Ok(())
    }

    /// Called by the Minecraft Process (Minecraft Plugin) as soon as the service has been fully started
    pub async fn service_notify_started(
        cloud: Arc<RwLock<Cloud>>,
        request: ServiceIdRequest,
    ) -> CloudResult<()> {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        node_manager
            .on_local_service_registered(EntityId::from(&request))
            .await?;

        Ok(())
    }

    /// Returns all backend servers currently available online
    pub async fn get_online_backend_services(
        cloud: Arc<RwLock<Cloud>>,
    ) -> Option<Value> {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        let services = node_manager.get_online_backend_server().await;

        let response: Vec<ServiceInfoResponse> = services
            .into_iter()
            .map(|s| ServiceInfoResponse::from(&s))
            .collect();

        Utils::convert_to_json(&response)
    }

    /// Called when a player performs an action (e.g., server change)
    pub async fn player_action(
        cloud: Arc<RwLock<Cloud>>,
        request: PlayerActionRequest,
    ) -> CloudResult<()> {
        let player_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_player_manager()
        };

        player_manager.handle_action(request).await?;

        Ok(())
    }
}
