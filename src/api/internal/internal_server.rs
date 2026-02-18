use std::sync::Arc;
use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use tokio::sync::RwLock;

use crate::api::internal::node_service::NodeService;
use crate::cloud::Cloud;
use crate::{error, log_error, log_info};
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
                            web::resource("cloud/node/get_online_backend_server")
                                .route(web::get().to(NodeService::get_online_backend_server)),
                        )
                        .service(
                            web::resource("cloud/node/set_online_status")
                                .route(web::post().to(NodeService::set_online_status)),
                        )
                        .service(
                            web::resource("cloud/node/info_shutdown")
                                .route(web::post().to(NodeService::shutdown)),
                        )
                        .service(
                            web::resource("cloud/node/send_player_action")
                                .route(web::post().to(NodeService::send_player_action)),
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

