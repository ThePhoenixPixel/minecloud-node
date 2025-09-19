use actix_web::{HttpResponse, web};
use serde::Deserialize;

use crate::core::task::Task;
use crate::utils::logger::Logger;
use crate::utils::utils::Utils;
use crate::{log_info, log_warning};

pub struct ApiTask;

#[derive(Deserialize)]
pub struct TaskGetRequest {
    task_name: String,
}

#[derive(Deserialize)]
pub struct TaskDeleteRequest {
    task_name: String,
}

#[derive(Deserialize)]
pub struct TaskCreateRequest {
    task_name: String,
    software_type: String,
    software_name: String,
}

impl ApiTask {
    pub async fn get_all() -> HttpResponse {
        /*
        HttpResponse::Ok().json(match serde_json::to_string_pretty(&Task::get_task_all()) {
            Ok(value) => value,
            Err(e) => return HttpResponse::InternalServerError().json(e.to_string()),
        })*/
        HttpResponse::Ok().json(Task::get_task_all())
    }

    pub async fn get(req: web::Query<TaskGetRequest>) -> HttpResponse {
        if req.task_name.is_empty() {
            return HttpResponse::NoContent().json("Bitte gebe ein task_name an");
        }

        let task = match Task::get_task(&req.task_name) {
            Some(task) => task,
            None => {
                return HttpResponse::NoContent().json("Bitte gebe ein Gültigen task_name an");
            }
        };

        match Utils::convert_to_json(&task) {
            Some(data) => HttpResponse::Ok().json(data),
            None => HttpResponse::InternalServerError()
                .json("Task konnte nicht in Json umgewandelt werden"),
        }
    }

    pub async fn create(req: web::Json<TaskCreateRequest>) -> HttpResponse {
        if req.task_name.is_empty() {
            return HttpResponse::BadRequest().json("Empty Task Name");
        }

        if req.software_type.is_empty() {
            return HttpResponse::BadRequest().json("Empty Software Type");
        }

        if req.software_name.is_empty() {
            return HttpResponse::BadRequest().json("Empty Software Name");
        }

        match Task::create(&req.task_name, &req.software_type, &req.software_name) {
            Ok(task) => {
                log_info!(
                    "[RestAPI] Task | {} | Erfolgreich erstellt",
                    task.get_name()
                );
                HttpResponse::Ok()
                    .json(format!("Task | {} | erfolgreich erstellt", task.get_name()))
            }
            Err(e) => {
                log_warning!(
                    "[RestAPI] Fehler beim ersttellen der Task {}",
                    req.task_name
                );
                HttpResponse::NotFound().json(e)
            }
        }
    }

    pub async fn change(req: web::Json<Task>) -> HttpResponse {
        if req.get_name().is_empty() {
            return HttpResponse::BadRequest().json("Empty Task Name");
        }

        if Task::get_task(req.get_name()).is_none() {
            return HttpResponse::BadRequest().json("Task Exsitiert nicht");
        }

        req.save_to_file();

        log_info!("[RestAPI] Task | {} | würde bearbeitet", req.get_name());
        HttpResponse::Ok().json("Task erfolgreich bearbeitet")
    }

    pub async fn delete(req: web::Query<TaskDeleteRequest>) -> HttpResponse {
        if req.task_name.is_empty() {
            return HttpResponse::BadRequest().json("Empty Task Name");
        }

        match Task::get_task(&req.task_name) {
            Some(task) => {
                task.delete_as_file();
                log_info!(
                    "[RestAPI] Task | {} | wurde erfolgreich gelöscht",
                    task.get_name()
                );
                HttpResponse::Ok().json("Task wurde erfolgreich gelöscht")
            }
            None => {
                log_warning!("[RestAPI] Kein Task gefunden | {} |", req.task_name);
                HttpResponse::BadRequest().json("Kein Task zum löschen gefunden")
            }
        }
    }
}
