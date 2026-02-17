use actix_web::{HttpResponse, web};
use serde::Deserialize;

use crate::types::task::Task;
use crate::utils::utils::Utils;
use crate::utils::log::logger::Logger;
use crate::{log_info, log_warning};

pub struct ApiTask;

#[derive(Deserialize)]
pub struct TaskNameRequest {
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
        HttpResponse::Ok().json(Task::get_task_all())
    }

    pub async fn get(req: web::Query<TaskNameRequest>) -> HttpResponse {
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

    pub async fn update(param: web::Query<TaskNameRequest>, req: web::Json<Task>) -> HttpResponse {
        if param.task_name.is_empty() || req.get_name().is_empty() {
            return HttpResponse::BadRequest().json("Empty Task Name");
        }

        let mut tasks = Task::get_task_all();
        match tasks.iter_mut().find(|t| t.get_name() == param.task_name) {
            Some(task) => {
                task.update(req.into_inner());
                log_info!("[RestAPI] Task | {} | wurde bearbeitet", task.get_name());
                HttpResponse::Ok().json(task)
            }
            None => HttpResponse::BadRequest().json("Task existiert nicht"),
        }
    }

    pub async fn delete(req: web::Query<TaskNameRequest>) -> HttpResponse {
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
