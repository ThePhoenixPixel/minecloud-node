use bx::network::address::Address;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::log_error;
use crate::types::{EntityId, PlayerAction, Service};

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    #[serde(rename = "request_id")]
    request_id: Option<Uuid>,

    #[serde(rename = "type")]
    msg_type: IncomingMessageType,

    #[serde(rename = "service_id")]
    service_id: Uuid,

    #[serde(default)]
    #[serde(rename = "data")]
    data: Value,
}

impl IncomingMessage {
    pub fn get_msg_typ(&self) -> &IncomingMessageType {
        &self.msg_type
    }

    pub fn get_request_id(&self) -> Option<Uuid> {
        self.request_id
    }

    pub fn get_service_id(&self) -> Uuid {
        self.service_id
    }

    pub fn get_data(&self) -> &Value {
        &self.data
    }
}

#[derive(Debug, Serialize)]
pub struct OutgoingMessage {
    #[serde(rename = "request_id")]
    request_id: Option<Uuid>,

    #[serde(rename = "type")]
    msg_type: OutgoingMessageType,

    // old
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "data")]
    data: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "error")]
    error: Option<String>,
}

impl OutgoingMessage {
    pub fn ok(
        request_id: Option<Uuid>,
        msg_type: impl Into<OutgoingMessageType>,
        data: Value,
    ) -> OutgoingMessage {
        OutgoingMessage {
            msg_type: msg_type.into(),
            request_id,
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(request_id: Option<Uuid>, error: String) -> OutgoingMessage {
        OutgoingMessage {
            msg_type: OutgoingMessageType::Error,
            request_id,
            success: false,
            data: None,
            error: Some(error),
        }
    }

    pub fn null(request_id: Option<Uuid>) -> OutgoingMessage {
        OutgoingMessage {
            msg_type: OutgoingMessageType::ResponseNull,
            request_id,
            success: true,
            data: None,
            error: None,
        }
    }

    pub fn set_request_id(&mut self, request_id: Option<Uuid>) {
        self.request_id = request_id;
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap_or_else(|e| {
            log_error!("CantSerializeOutgoingMsg: {}", e);
            String::from("CantSerializeOutgoingMsg Internal Server Error")
        })
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum IncomingMessageType {
    #[serde(rename = "auth")]
    Auth,

    #[serde(rename = "get_online_backend_service")]
    GetOnlineBackendServices,

    #[serde(rename = "service_notify_online")]
    ServiceOnline,

    #[serde(rename = "shutdown")]
    Shutdown,

    #[serde(rename = "player_action")]
    PlayerAction,
}

impl PartialEq<IncomingMessageType> for &IncomingMessageType {
    fn eq(&self, other: &IncomingMessageType) -> bool {
        **self == *other
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum OutgoingMessageType {
    #[serde(rename = "error")]
    Error,

    #[serde(rename = "response")]
    Response,

    #[serde(rename = "response_null")]
    ResponseNull,

    #[serde(rename = "shutdown")]
    Shutdown,

    #[serde(rename = "register_server")]
    AddServer,

    #[serde(rename = "remove_server")]
    RemoveServer,

    #[serde(rename = "connect_player_to_server")]
    ConnectPlayerToServer,
}

impl PartialEq<OutgoingMessageType> for &OutgoingMessageType {
    fn eq(&self, other: &OutgoingMessageType) -> bool {
        **self == *other
    }
}

#[derive(Deserialize)]
pub struct ServiceIdRequest {
    id: Uuid,
}

impl From<&ServiceIdRequest> for EntityId {
    fn from(value: &ServiceIdRequest) -> Self {
        value.id
    }
}

impl From<&Uuid> for ServiceIdRequest {
    fn from(value: &Uuid) -> Self {
        ServiceIdRequest { id: value.clone() }
    }
}

#[derive(Serialize, Debug)]
pub struct ServiceInfoResponse {
    #[serde(rename = "id")]
    id: Uuid,

    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "address")]
    address: Address,

    #[serde(rename = "join_permission")]
    join_permission: String,
}

impl ServiceInfoResponse {
    pub fn new(service: &Service) -> ServiceInfoResponse {
        ServiceInfoResponse {
            id: service.get_id().clone(),
            name: service.get_name().to_string(),
            address: service.get_server_listener().clone(),
            join_permission: service.get_join_permission().to_string(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_address(&self) -> Address {
        self.address.clone()
    }

    pub fn get_join_permission(&self) -> String {
        self.join_permission.clone()
    }
}

impl From<&Service> for ServiceInfoResponse {
    fn from(service: &Service) -> ServiceInfoResponse {
        ServiceInfoResponse::new(service)
    }
}

impl From<Service> for ServiceInfoResponse {
    fn from(service: Service) -> ServiceInfoResponse {
        ServiceInfoResponse::new(&service)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerActionMessage {
    #[serde(rename = "action")]
    action: PlayerAction,

    #[serde(rename = "service_uuid")]
    service_uuid: Uuid,

    #[serde(rename = "service_name")]
    service_name: String,

    #[serde(rename = "player_uuid")]
    player_uuid: Uuid,

    #[serde(rename = "player_name")]
    player_name: String,
}

impl PlayerActionMessage {
    pub fn new(action: PlayerAction, service_uuid: Uuid, service_name: String, player_uuid: Uuid, player_name: String) -> PlayerActionMessage {
        PlayerActionMessage {
            action,
            service_uuid,
            service_name,
            player_uuid,
            player_name,
        }
    }
    pub fn get_action(&self) -> &PlayerAction {
        &self.action
    }

    pub fn get_service_uuid(&self) -> Uuid {
        self.service_uuid
    }

    pub fn get_service_name(&self) -> &str {
        &self.service_name
    }

    pub fn get_player_name(&self) -> &str {
        &self.player_name
    }

    pub fn get_player_uuid(&self) -> Uuid {
        self.player_uuid
    }
}
