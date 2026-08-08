use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use futures_util::StreamExt;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::internal::{
    APIInternalHandler, IncomingMessage, IncomingMessageType, OutgoingMessage, OutgoingMessageType,
    PlayerActionMessage,
};
use crate::cloud::Cloud;
use crate::types::{EntityId, ServiceProcessRef};
use crate::utils::error::{CantBindAddress, CloudResult, IntoCloudError};
use crate::{error, log_error, log_info, log_warning};

async fn ws_handler(
    req: HttpRequest,
    body: web::Payload,
    cloud: web::Data<Arc<RwLock<Cloud>>>,
) -> actix_web::Result<HttpResponse> {
    let (response, session, stream) = actix_ws::handle(&req, body)?;
    let cloud = cloud.get_ref().clone();

    actix_web::rt::spawn(async move {
        handle_connection(cloud, session, stream).await;
    });

    Ok(response)
}

async fn handle_connection(
    cloud: Arc<RwLock<Cloud>>,
    mut session: Session,
    mut stream: impl StreamExt<Item = Result<Message, actix_ws::ProtocolError>> + Unpin,
) {
    let mut bound_service: Option<ServiceProcessRef> = None;

    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(text) => {
                let incoming: IncomingMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        let _ = session.text(e.to_string()).await;
                        continue;
                    }
                };

                let service_process_ref = {
                    let sm = cloud.read().await.get_node_manager().get_service_manager();
                    sm.read().await.find_from_id(&incoming.get_service_id())
                };

                bound_service = service_process_ref.clone();

                if bound_service.is_none() {
                    let _ = session
                        .text(
                            OutgoingMessage::err(
                                incoming.get_request_id(),
                                format!("Cant find Service: {}", incoming.get_service_id()),
                            )
                            .to_string(),
                        )
                        .await;
                    continue;
                }

                if incoming.get_msg_typ() == IncomingMessageType::Auth {
                    match service_process_ref {
                        Some(spr) => {
                            spr.write().await.attach_session(session.clone());

                            log_info!(4, "[API] Server '{}' Auth", incoming.get_service_id());

                            let response = OutgoingMessage::ok(
                                incoming.get_request_id(),
                                OutgoingMessageType::Response,
                                json!({
                                    "success": true
                                }),
                            );

                            if session.text(response.to_string()).await.is_err() {
                                log_warning!(
                                    6,
                                    "Cant send Auth response to '{}'",
                                    incoming.get_service_id()
                                );
                            }
                        }

                        None => {
                            let response = OutgoingMessage::err(
                                incoming.get_request_id(),
                                format!("Unknown service: '{}'", incoming.get_service_id()),
                            );

                            let _ = session.text(response.to_string()).await;
                            let _ = session.close(None).await;

                            return;
                        }
                    }

                    continue;
                }

                // Normales Message-Routing
                let msg = handle_text_message(incoming, cloud.clone())
                    .await
                    .to_string();

                if session.text(msg).await.is_err() {
                    log_warning!(6, "Cant send WS Answer");
                    break;
                }
            }

            Message::Ping(b) => {
                let _ = session.pong(&b).await;
            }

            Message::Close(reason) => {
                if let Some(svc) = &bound_service {
                    svc.write().await.detach_session();
                    log_info!(4, "[API] Server '{}' disconnected", svc.get_name().await);
                }
                let _ = session.close(reason).await;
                return;
            }

            _ => {}
        }
    }

    if let Some(svc) = &bound_service {
        svc.write().await.detach_session();
        log_info!(3, "[API] Server '{}' lost connection", svc.get_name().await);
    }
}

async fn handle_text_message(msg: IncomingMessage, cloud: Arc<RwLock<Cloud>>) -> OutgoingMessage {
    let mut result = match msg.get_msg_typ() {
        IncomingMessageType::GetOnlineBackendServices => {
            APIInternalHandler::get_online_backend_services(cloud).await
        }

        IncomingMessageType::ServiceOnline => {
            APIInternalHandler::service_notify_started(cloud, EntityId::from(msg.get_service_id()))
                .await
        }

        IncomingMessageType::Shutdown => {
            APIInternalHandler::service_notify_shutdown(cloud, EntityId::from(msg.get_service_id()))
                .await
        }

        IncomingMessageType::PlayerAction => {
            match serde_json::from_value::<PlayerActionMessage>(
                msg.get_data().clone()
            ) {
                Ok(data) => {
                    APIInternalHandler::player_action(cloud, data).await
                }

                Err(e) => {
                    log_error!(3, "[PlayerAction] Invalid request data: {}", e);

                    OutgoingMessage::err(
                        None,
                        format!("Invalid PlayerAction data: {}", e),
                    )
                }
            }
        }
        _ => OutgoingMessage::err(None, "Unknown message type".to_string()),
    };

    result.set_request_id(msg.get_request_id());
    result
}

pub struct APIInternal;

impl APIInternal {
    pub async fn start(cloud: Arc<RwLock<Cloud>>) -> CloudResult<()> {
        log_info!(3, "Start Internal WebSocket Server");

        let config = {
            let c = cloud.read().await;
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
                        .route("/internal", web::get().to(ws_handler))
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
                    log_error!("Internal WS Server Error: {}", e);
                }
            });
        });

        rx.recv().unwrap_or(Err(error!(CantBindAddress)))?;

        log_info!(3, "[Internal API] Endpoint started");
        Ok(())
    }
}
