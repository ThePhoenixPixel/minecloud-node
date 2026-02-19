use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::{error, log_error, log_info};
use crate::api::internal::APIInternalHandler;
use crate::utils::error::{CantBindAddress, CloudResult, IntoCloudError};

pub struct APIInternal;

impl APIInternal {

    pub async fn start(cloud: Arc<RwLock<Cloud>>) -> CloudResult<()> {
        log_info!(3, "Start Internal API Point");
        let config = {
            let cp = cloud.clone();
            let c = cp.read().await;
            c.get_config().clone()
        };

        let bind_addr = config.get_node_host().to_string();

        let (tx, rx) = std::sync::mpsc::channel::<CloudResult<()>>();

        std::thread::spawn(move || {
            let system = actix_web::rt::System::new();
            system.block_on(async move {
                let app = move || {
                    App::new()
                        .app_data(web::Data::new(cloud.clone()))
                        .wrap(
                            Cors::permissive()
                                .allow_any_method()
                                .allow_any_header()
                                .supports_credentials(),
                        )
                        .service(
                            web::resource("api/internal/services/backend")
                                .route(web::get().to(APIInternalHandler::get_backend_services)),
                        )
                        .service(
                            web::resource("api/internal/service/online")
                                .route(web::post().to(APIInternalHandler::service_set_online)),
                        )
                        .service(
                            web::resource("api/internal/service/shutdown")
                                .route(web::post().to(APIInternalHandler::service_notify_shutdown)),
                        )
                        .service(
                            web::resource("api/internal/player/action")
                                .route(web::post().to(APIInternalHandler::player_action)),
                        )
                };

                let server = match HttpServer::new(app).bind(&bind_addr) {
                    Ok(s) => {
                        let _ = tx.send(Ok(()));
                        s
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e).into_cloud_error(CantBindAddress));
                        return;
                    }
                };

                if let Err(e) = server.run().await {
                    log_error!("Internal API Server error: {}", e);
                }
            });
        });

        rx.recv().unwrap_or(Err(error!(CantBindAddress)))?;

        log_info!(3, "[Internal API] Server gestartet!");
        Ok(())
    }
}

