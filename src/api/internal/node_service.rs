use actix_web::{HttpResponse, web};
use bx::network::address::Address;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cloud::Cloud;
use crate::{log_error, log_info, log_warning};
use crate::types::{PlayerAction, PlayerRequest, ServiceStatus, Service};
use crate::utils::utils::Utils;

#[derive(Deserialize)]
pub struct OnlineStatusRequest {
    id: Uuid,
}

#[derive(Deserialize)]
pub struct ShutdownRequest {
    id: Uuid,
}

#[derive(Serialize, Debug)]
pub struct ServiceInfoResponse {
    name: String,
    address: Address,
    default_connect: bool,
    join_permission: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerActionRequest {
    action: PlayerAction,
    service_uuid: Uuid,
    player: PlayerRequest,
}

#[derive(Deserialize)]
pub struct NodeService;

impl NodeService {
    pub async fn shutdown(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<ShutdownRequest>,
    ) -> HttpResponse {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };
        todo!();
        /*let service = {
            match service_manager.read().await.get_from_id(&request.id) {
                Some(service) => service.get_service().clone(),
                None => return HttpResponse::NoContent().json("Service not found"),
            }
        };
        let service_name = service.get_name();
        if let Err(e) = service_manager.read().await
            .disconnect_from_network(&service)
            .await
        {
            log_error!(
                "Service: {} konnte nicht vom Netzwerk getrennt werden \n Error: {}",
                service_name,
                e
            );
            return HttpResponse::InternalServerError().json(format!(
                "Service: {} not disconnected from Network \n Error: {}",
                service_name, e
            ));
        }

        {
            let service_id = service.get_id();
            tokio::spawn(async move {
                Utils::wait_sec(service.get_task().get_time_shutdown_before_kill().as_secs()).await;

                if let Some((pos, s)) = service_manager.write().await.get_from_id_mut(&service_id) {

                    if let Err(e) = s.kill().await {
                        log_warning!(
                            "Service [{}] konnte nicht gekillt werden: {}",
                            s.get_service().get_name(),
                            e
                        );
                    } else {
                        log_info!("Service [{}] wurde gekillt", s.get_service().get_name());
                    }

                    {
                        service_manager.write().await.remove_service(pos)
                    }

                } else {
                    log_error!(
                        "Konnte Stop-Status für Service [{}] nicht setzen",
                        service_id
                    );
                }
            });
        }

        HttpResponse::Ok().json(format!(
            "Service: {} successfully disconnected from Network",
            service_name
        ))*/
    }

    pub async fn set_online_status(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<OnlineStatusRequest>,
    ) -> HttpResponse {
        let service_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };
todo!();
        /*
        let service = {
            match service_manager.write().await.get_from_id(&request.id) {
                Some(mut service) => {
                    service.get_service_mut().set_status(ServiceStatus::Running);
                    service.get_service_mut().save_to_file();
                    service.get_service().clone()
                },
                None => return HttpResponse::NoContent().json("Service not found"),
            }
        };
        match service_manager.read().await.connect_to_network(&service).await {
            Ok(_) => HttpResponse::Ok().json(format!(
                "Service: {} successfully connect to Network",
                service.get_name()
            )),

            Err(e) => {
                log_error!(
                    "Service: {} not connect to Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                );
                HttpResponse::InternalServerError().json(format!(
                    "Service: {} not connect to Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                ))
            }
        }
        */

    }

    pub async fn get_online_backend_server(cloud: web::Data<Arc<RwLock<Cloud>>>) -> HttpResponse {
        let node_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_node_manager()
        };

        let all_service = node_manager.get_online_backend_server().await;

        let response: Vec<ServiceInfoResponse> = all_service
            .into_iter()
            .map(|s| ServiceInfoResponse::new(&s))
            .collect();

        HttpResponse::Ok().json(response)
    }

    pub async fn send_player_action(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<PlayerActionRequest>,
    ) -> HttpResponse {
        let player_manager = {
            let cloud_guard = cloud.read().await;
            cloud_guard.get_player_manager()
        };
        todo!();
        /*
        match player_manager.handle_action(request.into_inner()).await{
            Ok(()) => HttpResponse::Ok().finish(),
            Err(e) => {
                log_error!("[Node-API] Cant Execute Player Action Request {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }*/
    }
}

impl PlayerActionRequest {
    pub fn get_action(&self) -> PlayerAction {
        self.action.clone()
    }
    pub fn get_player_req(&self) -> PlayerRequest {
        self.player.clone()
    }
    pub fn get_service_uuid(&self) -> Uuid {
        self.service_uuid.clone()
    }
}

impl ServiceInfoResponse {
    pub fn new(service: &Service) -> ServiceInfoResponse {
        ServiceInfoResponse {
            name: service.get_name(),
            address: service.get_server_listener(),
            default_connect: service.get_task().default_connect(),
            join_permission: service.get_task().get_join_permission().to_string(),
        }
    }
}
