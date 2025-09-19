use actix_web::{HttpResponse, web};
use bx::network::address::Address;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cloud::Cloud;
use crate::core::service::Service;
use crate::utils::logger::Logger;
use crate::utils::service_status::ServiceStatus;
use crate::log_error;

#[derive(Deserialize)]
pub struct OnlineStatusRequest {
    name: String,
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
        cloud_org: web::Data<Arc<Mutex<Cloud>>>,
        request: web::Query<OnlineStatusRequest>,
    ) -> HttpResponse {
        let service = {
            let mut cloud = cloud_org.lock().await;
            match cloud.get_local_mut().get_from_name_mut(&request.name) {
                Some(service) => {
                    service.set_status(ServiceStatus::Stop);
                    service
                },
                None => return HttpResponse::NoContent().json("Service not found"),
            }.clone_without_process()
        };

        match service.disconnect_from_network(cloud_org.get_ref().clone()).await {
            Ok(_) => HttpResponse::Ok().json(format!(
                "Service: {} successfully disconnect from Network",
                service.get_name()
            )),
            Err(e) => {
                log_error!(
                    "Service: {} not disconnect from Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                );
                HttpResponse::InternalServerError().json(format!(
                    "Service: {} not disconnect from Network \n Error: {}",
                    service.get_name(),
                    e.to_string()
                ))
            }
        }
    }

    pub async fn set_online_status(
        cloud_org: web::Data<Arc<Mutex<Cloud>>>,
        request: web::Query<OnlineStatusRequest>,
    ) -> HttpResponse {
        let mut cloud = cloud_org.lock().await;
        let service = match cloud.get_local_mut().get_from_name_mut(&request.name) {
            Some(service) => service,
            None => return HttpResponse::NoContent().json("Service not found"),
        };

        match service.connect_to_network(cloud_org.get_ref().clone()).await {
            Ok(_) => {
                service.set_status(ServiceStatus::Start);
                HttpResponse::Ok().json(format!(
                    "Service: {} successfully connect to Network",
                    service.get_name()
                ))
            },
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

    pub async fn get_online_backend_server(
        cloud: web::Data<Arc<Mutex<Cloud>>>,
    ) -> HttpResponse {
        let cloud = cloud.lock().await;
        let services = cloud.get_all().get_online_backend_services().await;

        let response: Vec<ServiceInfoResponse> = services
            .into_iter()
            .map(|s| ServiceInfoResponse::new(&s))
            .collect();

        HttpResponse::Ok().json(response)
    }
}

impl ServiceInfoResponse {
    pub fn new(service: &Service) -> ServiceInfoResponse {
        ServiceInfoResponse {
            name: service.get_name(),
            address: service.get_server_address(),
            default_connect: service.get_task().default_connect(),
            join_permission: service.get_task().get_join_permission().to_string(),
        }
    }
}
