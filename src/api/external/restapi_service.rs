use actix_web::{HttpResponse, web};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::cloud::Cloud;
use crate::types::task::Task;

pub struct ApiService;

#[derive(Deserialize)]
pub struct ServiceGetRequest {
    id: Uuid,
}

#[derive(Deserialize)]
pub struct ServiceCreateRequest {
    task_name: String,
}

impl ApiService {
    pub async fn get_all(cloud: web::Data<Arc<RwLock<Cloud>>>) -> HttpResponse {
        let all_services = { cloud.read().await.get_all().clone() };
        let services = all_services.get_all().await;
        HttpResponse::Ok().json(services)
    }

    pub async fn get_online(cloud: web::Data<Arc<RwLock<Cloud>>>) -> HttpResponse {
        let all_services = { cloud.read().await.get_all().clone() };
        let services = all_services.get_start_services().await;
        HttpResponse::Ok().json(services)
    }

    pub async fn get_prepare(cloud: web::Data<Arc<RwLock<Cloud>>>) -> HttpResponse {
        let all_services = { cloud.read().await.get_all().clone() };
        let services = all_services.get_prepare_services().await;
        HttpResponse::Ok().json(services)
    }

    pub async fn get_offline(cloud: web::Data<Arc<RwLock<Cloud>>>) -> HttpResponse {
        let all_services = { cloud.read().await.get_all().clone() };
        let services = all_services.get_stopped_services().await;
        HttpResponse::Ok().json(services)
    }

    pub async fn get_from_id(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        req: web::Query<ServiceGetRequest>,
    ) -> HttpResponse {
        if req.id.is_nil() {
            return HttpResponse::NoContent().json("Bitte gebe ein Service ID an");
        }

        let all_service = { cloud.read().await.get_all().clone() };

        let service = match all_service.get_from_id(&req.id).await {
            Some(service) => service,
            None => {
                return HttpResponse::NoContent().json("Bitte gebe ein Gültige ID an");
            }
        };

        HttpResponse::Ok().json(service)
    }

    pub async fn create(
        cloud: web::Data<Arc<RwLock<Cloud>>>,
        req: web::Json<ServiceCreateRequest>,
    ) -> HttpResponse {
        if req.task_name.is_empty() {
            return HttpResponse::NoContent().json("Bitte gebe ein task_name an");
        }

        let task = match Task::get_task(&req.task_name) {
            Some(task) => task,
            None => return HttpResponse::NotFound().json("Task nicht gefunden!!!"),
        };

        let result = {
            let mut cloud = cloud.write().await;
            cloud.get_all_mut().start_service(&task).await
        };

        match result {
            Ok(_) => HttpResponse::Ok().json("Service gestartet"),
            Err(e) => {
                HttpResponse::InternalServerError().json(format!("Fehler beim Starten: {}", e))
            }
        }
    }
}
