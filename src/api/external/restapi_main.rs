use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::api::external::restapi_service::ApiService;
use crate::api::external::restapi_task::ApiTask;
use crate::config::cloud_config::CloudConfig;
use crate::utils::log::logger::Logger;
use crate::{log_error, log_info, log_warning};

pub struct ApiMain;

impl ApiMain {
    #[actix_web::main]
    pub async fn start(cloud: Arc<RwLock<Cloud>>) {
        log_info!("Start the REST AIP Server");
        let app_factory = move || {
            App::new()
                .app_data(web::Data::new(cloud.clone()))
                .wrap(
                    Cors::permissive()
                        .allow_any_method()
                        .supports_credentials()
                        .allow_any_header(),
                )
                // Task
                .service(web::resource("cloud/task/get_all").route(web::get().to(ApiTask::get_all)))
                .service(web::resource("cloud/task/get").route(web::get().to(ApiTask::get)))
                .service(web::resource("cloud/task/create").route(web::post().to(ApiTask::create)))
                .service(web::resource("cloud/task/update").route(web::put().to(ApiTask::update)))
                .service(
                    web::resource("cloud/task/delete").route(web::delete().to(ApiTask::delete)),
                )
                // Service
                .service(
                    web::resource("cloud/service/all").route(web::get().to(ApiService::get_all)),
                )
                .service(
                    web::resource("cloud/service/online")
                        .route(web::get().to(ApiService::get_online)),
                )
                .service(
                    web::resource("cloud/service/prepared")
                        .route(web::get().to(ApiService::get_prepare)),
                )
                .service(
                    web::resource("cloud/service/offline")
                        .route(web::get().to(ApiService::get_offline)),
                )
                .service(
                    web::resource("cloud/service/get/")
                        .route(web::get().to(ApiService::get_from_id)),
                )
                .service(
                    web::resource("cloud/service/create").route(web::post().to(ApiService::create)),
                )
        };

        // bind the address
        let http_server = match HttpServer::new(app_factory)
            .bind(CloudConfig::get().get_rest_api().to_string())
        {
            Ok(http_server) => http_server,
            Err(e) => {
                log_warning!(1,
                    "Can not bind the REST API Server at {}",
                    CloudConfig::get().get_rest_api().to_string()
                );
                log_error!("{}", e.to_string());
                return;
            }
        };

        // start the server
        match http_server.run().await {
            Ok(_) => log_info!("Rest Api Server successfully start"),
            Err(e) => {
                log_error!("{}", e.to_string());
                return;
            }
        }
    }
}
