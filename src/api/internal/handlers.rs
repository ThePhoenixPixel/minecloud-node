use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::internal::{
    OutgoingMessage, OutgoingMessageType, PlayerActionResponse, ServiceInfoResponse,
};
use crate::cloud::Cloud;
use crate::log_error;
use crate::types::EntityId;
use crate::utils::utils::Utils;

pub struct APIInternalHandler;

impl APIInternalHandler {
    /// Called by the Minecraft Process (Minecraft Plugin) when a service is shutdown
    pub async fn service_notify_shutdown(
        cloud: Arc<RwLock<Cloud>>,
        service_id: EntityId,
    ) -> OutgoingMessage {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        match node_manager.on_local_service_shutdown(service_id).await {
            Ok(()) => OutgoingMessage::null(None),
            Err(e) => {
                log_error!(3, "[service_notify_shutdown] Error: {}", e);
                OutgoingMessage::err(None, e.to_string())
            }
        }
    }

    /// Called by the Minecraft Process (Minecraft Plugin) as soon as the service has been fully started
    pub async fn service_notify_started(
        cloud: Arc<RwLock<Cloud>>,
        service_id: EntityId,
    ) -> OutgoingMessage {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        match node_manager.on_local_service_registered(service_id).await {
            Ok(()) => OutgoingMessage::null(None),
            Err(e) => {
                log_error!(3, "[service_notify_started] Error: {}", e);
                OutgoingMessage::err(None, e.to_string())
            }
        }
    }

    /// Returns all backend servers currently available online
    pub async fn get_online_backend_services(cloud: Arc<RwLock<Cloud>>) -> OutgoingMessage {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        let services = node_manager.get_online_backend_server().await;

        let response: Vec<ServiceInfoResponse> = services
            .into_iter()
            .map(|s| ServiceInfoResponse::from(&s))
            .collect();

        match Utils::convert_to_json(&response) {
            Some(data) => OutgoingMessage::ok(None, OutgoingMessageType::Response, data),
            None => OutgoingMessage::err(None, "Cant Serialize Data".to_string()),
        }
    }

    /// Called when a player performs an action (e.g., server change)
    pub async fn player_action(
        cloud: Arc<RwLock<Cloud>>,
        request: PlayerActionResponse,
    ) -> OutgoingMessage {
        let player_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_player_manager()
        };

        player_manager
            .handle_action(request)
            .await
            .unwrap_or_else(|e| {
                log_error!("{}", e);
                OutgoingMessage::err(None, e.to_string())
            })
    }
}
