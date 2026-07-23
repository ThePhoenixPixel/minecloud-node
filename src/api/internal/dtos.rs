use bx::network::address::Address;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::log_error;
use crate::types::{EntityId, PlayerAction, Service};

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    #[serde(rename = "type")]
    msg_type: IncomingMessageType,

    service_id: Uuid,

    #[serde(default)]
    data: Value,
}

impl IncomingMessage {
    pub fn get_msg_typ(&self) -> &IncomingMessageType {
        &self.msg_type
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
    #[serde(rename = "type")]
    msg_type: OutgoingMessageType,

    // old
    success: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl OutgoingMessage {
    pub fn ok(msg_type: impl Into<OutgoingMessageType>, data: Value) -> OutgoingMessage {
        OutgoingMessage {
            msg_type: msg_type.into(),
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> OutgoingMessage {
        OutgoingMessage {
            msg_type: OutgoingMessageType::Error,
            success: true,
            data: None,
            error: Some(error),
        }
    }

    pub fn null() -> OutgoingMessage {
        OutgoingMessage {
            msg_type: OutgoingMessageType::ResponseNull,
            success: true,
            data: None,
            error: None,
        }
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
    #[serde(rename = "Auth")]
    Auth,

    #[serde(rename = "get_online_backend_services")]
    GetOnlineBackendServices,

    #[serde(rename = "service_online")]
    ServiceOnline,

    #[serde(rename = "service_shutdown")]
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

    #[serde(rename = "add_server")]
    AddServer,

    #[serde(rename = "remove_server")]
    RemoveServer,

    #[serde(rename = "")]
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

#[derive(Serialize, Debug)]
pub struct ServiceInfoResponse {
    name: String,
    address: Address,
    default_connect: bool,
    join_permission: String,
}

impl ServiceInfoResponse {
    pub fn new(service: &Service) -> ServiceInfoResponse {
        ServiceInfoResponse {
            name: service.get_name().to_string(),
            address: service.get_server_listener().clone(),
            default_connect: service.default_connect(),
            join_permission: service.get_join_permission().to_string(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_address(&self) -> Address {
        self.address.clone()
    }

    pub fn is_default_connect(&self) -> bool {
        self.default_connect
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
pub struct PlayerActionRequest {
    action: PlayerAction,
    service_uuid: Uuid,
    player_uuid: Uuid,
    player_name: String,
}

impl PlayerActionRequest {
    pub fn get_action(&self) -> &PlayerAction {
        &self.action
    }

    pub fn get_player_name(&self) -> &str {
        &self.player_name
    }

    pub fn get_player_uuid(&self) -> Uuid {
        self.player_uuid
    }

    pub fn get_service_uuid(&self) -> Uuid {
        self.service_uuid
    }
}
