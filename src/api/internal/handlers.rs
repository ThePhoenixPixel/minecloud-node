use actix_web::{HttpResponse, web};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::api::internal::{PlayerActionRequest, ServiceIdRequest, ServiceInfoResponse};


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
    pub async fn service_shutdown(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<ServiceIdRequest>,
    ) -> HttpResponse {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };
        todo!()
    }

    /// POST /api/internal/service/online
    /// Called by the Minecraft Process (Minecraft Plugin) as soon as the service has been fully started
    pub async fn service_set_online(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<ServiceIdRequest>,
    ) -> HttpResponse {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };
        todo!()
    }

    /// GET /api/internal/services/backend
    /// Returns all backend servers currently available online
    pub async fn get_backend_services(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
    ) -> HttpResponse {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        let services = node_manager.get_online_backend_server().await;

        let response: Vec<ServiceInfoResponse> = services
            .into_iter()
            .map(|s| ServiceInfoResponse::from(&s))
            .collect();

        HttpResponse::Ok().json(response)
    }

    /// POST /api/internal/player/action
    /// Called when a player performs an action (e.g., server change)
    pub async fn player_action(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<PlayerActionRequest>,
    ) -> HttpResponse {
        let player_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_player_manager()
        };
        todo!()
    }
}