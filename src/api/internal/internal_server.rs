use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_ws::{Message, Session};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use futures_util::StreamExt;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::api::internal::{APIInternalHandler, PlayerActionRequest, ServiceIdRequest};
use crate::cloud::Cloud;
use crate::utils::error::{CantBindAddress, CloudResult, IntoCloudError};
use crate::{error, log_error, log_info};
use crate::types::{ServiceProcessRef};

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    /// z.B. "get_backend_services" | "service_online" | "service_shutdown" | "player_action"
    #[serde(rename = "type")]
    msg_type: String,

    service_id: Uuid,

    /// Body
    #[serde(default)]
    data: Value,
}

#[derive(Debug, Serialize)]
pub struct OutgoingMessage {
    #[serde(rename = "type")]
    msg_type: String,

    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl OutgoingMessage {
    fn ok(msg_type: impl Into<String>, data: Option<Value>) -> String {
        serde_json::to_string(&Self {
            msg_type: msg_type.into(),
            success: true,
            data,
            error: None,
        })
            .unwrap()
    }

    fn err(msg_type: impl Into<String>, e: impl ToString) -> String {
        serde_json::to_string(&Self {
            msg_type: msg_type.into(),
            success: false,
            data: None,
            error: Some(e.to_string()),
        })
            .unwrap()
    }
}

// ── WebSocket-Handler ────────────────────────────────────────────────────────

async fn ws_handler(
    req: HttpRequest,
    body: web::Payload,
    cloud: web::Data<Arc<RwLock<Cloud>>>,
) -> actix_web::Result<HttpResponse> {
    let (response, session, mut stream) = actix_ws::handle(&req, body)?;
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
                        let _ = session.text(OutgoingMessage::err("error", e)).await;
                        continue;
                    }
                };

                let service_process_ref = {
                    let sm = cloud.read().await.get_node_manager().get_service_manager();
                    sm.read().await.find_from_id(&incoming.service_id)
                };

                bound_service = service_process_ref.clone();

                if incoming.msg_type == "auth" {


                    match service_process_ref {
                        Some(spr) => {
                            spr.write().await.attach_session(session.clone());

                            log_info!(3, "[WS] Plugin '{}' auth", incoming.service_id);
                            let _ = session.text(OutgoingMessage::ok("auth", Some(json!({ "status": "ok" })))).await;
                        }
                        None => {
                            let _ = session.text(OutgoingMessage::err("auth", format!("Unknown service: '{}'", incoming.service_id))).await;
                            let _ = session.close(None).await;
                            return;
                        }
                    }
                    continue;
                }

                if bound_service.is_none() {
                    let _ = session.text(OutgoingMessage::err(
                        "error",
                        "Not identified yet. Send 'identify' first.",
                    )).await;
                    continue;
                }

                // Normales Message-Routing
                let reply = handle_text_message(incoming, cloud.clone()).await;
                if session.text(reply).await.is_err() {
                    break;
                }
            }

            Message::Ping(b) => { let _ = session.pong(&b).await; }

            Message::Close(reason) => {
                if let Some(svc) = &bound_service {
                    svc.write().await.detach_session();
                    log_info!(3, "[WS] Plugin '{}' disconnected", svc.get_name().await);
                }
                let _ = session.close(reason).await;
                return;
            }

            _ => {}
        }
    }

    // Cleanup falls Stream unerwartet endet
    if let Some(svc) = &bound_service {
        svc.write().await.detach_session();
        log_info!(3, "[WS] Plugin '{}' lost connection", svc.get_name().await);
    }
}

async fn handle_text_message(msg: IncomingMessage, cloud: Arc<RwLock<Cloud>>) -> String {
    let service_id = msg.service_id.to_string();

    match msg.msg_type.as_str() {
        // GET /api/internal/services/backend
        "get_online_backend_services" => {
            let data = APIInternalHandler::get_online_backend_services(cloud).await;
            OutgoingMessage::ok("get_online_backend_server", data)
        }

        // POST /api/internal/service/online
        "service_online" => {
            let data: ServiceIdRequest  = serde_json::from_value(msg.data.clone()).unwrap();
            match APIInternalHandler::service_set_online(cloud, data).await {
                Ok(_) => OutgoingMessage::ok("service_online", None),
                Err(e)   => OutgoingMessage::err("service_online", e),
            }
        }

        // POST /api/internal/service/shutdown
        "service_shutdown" => {
            let data: ServiceIdRequest  = serde_json::from_value(msg.data.clone()).unwrap();
            match APIInternalHandler::service_notify_shutdown(cloud, data).await {
                Ok(_) => OutgoingMessage::ok("service_shutdown", None),
                Err(e)   => OutgoingMessage::err("service_shutdown", e),
            }
        }

        // POST /api/internal/player/action
        "player_action" => {
            let data: PlayerActionRequest  = serde_json::from_value(msg.data.clone()).unwrap();
            match APIInternalHandler::player_action(cloud, data).await {
                Ok(_) => OutgoingMessage::ok("player_action", None),
                Err(e)   => OutgoingMessage::err("player_action", e),
            }
        }

        unknown => {
            OutgoingMessage::err("error", format!("Unknown message type: '{unknown}'"))
        }
    }
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
                    log_error!("Internal WS Server error: {}", e);
                }
            });
        });

        rx.recv().unwrap_or(Err(error!(CantBindAddress)))?;

        log_info!(3, "[Internal WS] Server gestartet");
        Ok(())
    }
}

