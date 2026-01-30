use actix_web::{HttpResponse, web};
use bx::network::address::Address;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cloud::Cloud;
use crate::core::service::Service;
use crate::api::internal::request::player_action_req::PlayerActionRequest;
use crate::utils::logger::Logger;
use crate::utils::service_status::ServiceStatus;
use crate::{log_error, log_info, log_warning};

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

#[derive(Deserialize)]
pub struct NodeService;

impl NodeService {
    pub async fn shutdown(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<ShutdownRequest>,
    ) -> HttpResponse {
        let service = {
            let cloud_guard = cloud.read().await;
            match cloud_guard.get_local().get_from_id(&request.id) {
                Some(service) => service,
                None => return HttpResponse::NoContent().json("Service not found"),
            }
        };
        let service_name = service.get_name();
        if let Err(e) = service
            .disconnect_from_network(cloud.get_ref().clone())
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
            let cloud_clone = cloud.clone();
            let service_id = service.get_id();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let mut cloud_guard = cloud_clone.write().await;
                if let Some(s) = cloud_guard.get_local_mut().get_from_id_mut(&service_id) {
                    s.set_status(ServiceStatus::Stop);

                    if s.get_process().is_some() {
                        if let Err(e) = s.kill() {
                            log_warning!(
                                "Service [{}] konnte nicht gekillt werden: {}",
                                s.get_name(),
                                e
                            );
                        } else {
                            log_info!("Service [{}] wurde gekillt", s.get_name());
                        }
                    }

                    if !service.get_task().is_static_service()
                        && service.get_task().is_delete_on_stop()
                    {
                        s.delete_files();
                        cloud_guard.get_local_mut().remove_service(service_id);
                    } else {
                        s.save_to_file();
                    }
                } else {
                    log_error!(
                        "Konnte Stop-Status für Service [{}] nicht setzen",
                        service_id
                    );
                }
                // check
                cloud_guard.get_all_mut().check_service().await;
            });
        }

        HttpResponse::Ok().json(format!(
            "Service: {} successfully disconnected from Network",
            service_name
        ))
    }

    pub async fn set_online_status(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<OnlineStatusRequest>,
    ) -> HttpResponse {
        let mut service = {
            let cloud = cloud.read().await;
            match cloud.get_local().get_from_id(&request.id) {
                Some(service) => service,
                None => return HttpResponse::NoContent().json("Service not found"),
            }
        };

        let service = {
            service.set_status(ServiceStatus::Start);
            service.save_to_file();
            let s = service.clone_without_process();
            cloud.write().await.get_local_mut().set_service(service);
            s
        };

        match service.connect_to_network(cloud.get_ref().clone()).await {
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
    }

    pub async fn get_online_backend_server(cloud: web::Data<Arc<RwLock<Cloud>>>) -> HttpResponse {
        let all_services = { cloud.read().await.get_all().clone() };

        let response: Vec<ServiceInfoResponse> = all_services
            .get_online_backend_services()
            .await
            .into_iter()
            .filter(|s| s.is_start() && s.is_backend_server())
            .map(|s| ServiceInfoResponse::new(&s))
            .collect();

        HttpResponse::Ok().json(response)
    }

    pub async fn send_player_action(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        request: web::Json<PlayerActionRequest>,
    ) -> HttpResponse {
        match request.execute(cloud.get_ref().clone()).await {
            Ok(()) => HttpResponse::Ok().finish(),
            Err(e) => {
                log_error!("[Node-API] Cant Execute Player Action Request {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
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
