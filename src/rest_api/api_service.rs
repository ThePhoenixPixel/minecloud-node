use crate::cloud::Cloud;
use crate::core::service::Service;
use crate::core::task::Task;
use crate::utils::utils::Utils;
use actix_web::{HttpResponse, web};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ApiService;

#[derive(Deserialize)]
pub struct ServiceGetRequest {
    service_name: String,
}

#[derive(Deserialize)]
pub struct ServiceCreateRequest {
    task_name: String,
}

impl ApiService {
    pub async fn get_all(cloud: web::Data<Arc<Mutex<Cloud>>>) -> HttpResponse {
        let cloud = cloud.lock().await;
        HttpResponse::Ok().json(match serde_json::to_string_pretty(&cloud.get_all().get_all().await) {
            Ok(value) => value,
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        })
    }

    pub async fn get_online(cloud: web::Data<Arc<Mutex<Cloud>>>) -> HttpResponse {
        let cloud = cloud.lock().await;
        HttpResponse::Ok().json(
            match serde_json::to_string_pretty(&cloud.get_all().get_start_services().await) {
                Ok(value) => value,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            },
        )
    }

    pub async fn get_prepare(cloud: web::Data<Arc<Mutex<Cloud>>>) -> HttpResponse {
        let cloud = cloud.lock().await;
        HttpResponse::Ok().json(
            match serde_json::to_string_pretty(&cloud.get_all().get_prepare_services().await) {
                Ok(value) => value,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            },
        )
    }

    pub async fn get_offline(cloud: web::Data<Arc<Mutex<Cloud>>>) -> HttpResponse {
        let cloud = cloud.lock().await;
        HttpResponse::Ok().json(
            match serde_json::to_string_pretty(&cloud.get_all().get_stop_services().await) {
                Ok(value) => value,
                Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
            },
        )
    }

    pub async fn get(req: web::Query<ServiceGetRequest>) -> HttpResponse {
        if req.service_name.is_empty() {
            return HttpResponse::NoContent().json("Bitte gebe ein service_name an");
        }

        let service = match Service::get_from_name(&req.service_name) {
            Some(service) => service,
            None => {
                return HttpResponse::NoContent().json("Bitte gebe ein Gültigen task_name an");
            }
        };

        match Utils::convert_to_json(&service) {
            Some(data) => HttpResponse::Ok().json(data),
            None => HttpResponse::InternalServerError()
                .json("Task konnte nicht in Json umgewandelt werden"),
        }
    }

    pub async fn create(
        cloud: web::Data<Arc<Mutex<Cloud>>>,
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
            let mut cloud = cloud.lock().await;
            cloud.get_all_mut().start_service(task).await
        };

        match result {
            Ok(_) => HttpResponse::Ok().json("Service gestartet"),
            Err(e) => HttpResponse::InternalServerError()
                .json(format!("Fehler beim Starten: {}", e)),
        }
    }
}
