use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cloud::Cloud;
use crate::api::internal::node_service::NodeService;
use crate::config::CloudConfig;
use crate::{log_error, log_info, log_warning};

pub struct NodeServer;

impl NodeServer {
    #[actix_web::main]
    pub async fn start(cloud: Arc<RwLock<Cloud>>) {
        log_info!("Start the Node Host");

        let app_factory = move || {
            App::new()
                .app_data(web::Data::new(cloud.clone()))
                .wrap(
                    Cors::permissive()
                        .allow_any_method()
                        .supports_credentials()
                        .allow_any_header(),
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

        // bind the address
        let http_server = match HttpServer::new(app_factory)
            .bind(CloudConfig::get().get_node_host().to_string())
        {
            Ok(http_server) => http_server,
            Err(e) => {
                log_warning!(
                    "Can not bind the NODE Server at {}",
                    CloudConfig::get().get_node_host().to_string()
                );
                log_error!("{}", e.to_string());
                return;
            }
        };

        // start the server
        match http_server.run().await {
            Ok(_) => log_info!("Node Server successfully start"),
            Err(e) => {
                log_error!("{}", e.to_string());
                return;
            }
        }
    }
}
