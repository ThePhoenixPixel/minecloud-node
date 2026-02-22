use actix_web::{HttpResponse, web};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::internal::{PlayerActionRequest, ServiceIdRequest, ServiceInfoResponse};
use crate::cloud::Cloud;
use crate::types::EntityId;
use crate::utils::error::CloudResult;

// ============================================================
//  GET  /api/internal/services/backend          → list all online Backend-Server
//  POST /api/internal/service/online            → Service reports itself as online
//  POST /api/internal/service/shutdown          → Service reports for shutdown
//  POST /api/internal/player/action             → Forward player action
// ============================================================

pub struct APIInternalHandler;

impl APIInternalHandler {
    /// POST /api/internal/service/shutdown
    /// Called by the Minecraft Process (Minecraft Plugin) when a service is shut down
    pub async fn service_notify_shutdown(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<ServiceIdRequest>,
    ) -> CloudResult<HttpResponse> {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        node_manager
            .on_local_service_shutdown(EntityId::from(&request.into_inner()))
            .await?;

        Ok(HttpResponse::Ok().finish())
    }

    /// POST /api/internal/service/online
    /// Called by the Minecraft Process (Minecraft Plugin) as soon as the service has been fully started
    pub async fn service_set_online(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<ServiceIdRequest>,
    ) -> CloudResult<HttpResponse> {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        node_manager
            .on_local_service_registered(EntityId::from(&request.into_inner()))
            .await?;

        Ok(HttpResponse::Ok().finish())
    }

    /// GET /api/internal/services/backend
    /// Returns all backend servers currently available online
    pub async fn get_backend_services(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
    ) -> CloudResult<HttpResponse> {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        let services = node_manager.get_online_backend_server().await;

        let response: Vec<ServiceInfoResponse> = services
            .into_iter()
            .map(|s| ServiceInfoResponse::from(&s))
            .collect();

        Ok(HttpResponse::Ok().json(response))
    }

    /// POST /api/internal/player/action
    /// Called when a player performs an action (e.g., server change)
    pub async fn player_action(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<PlayerActionRequest>,
    ) -> CloudResult<HttpResponse> {
        let player_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_player_manager()
        };

        player_manager.handle_action(request.into_inner()).await?;

        Ok(HttpResponse::Ok().finish())
    }
}
